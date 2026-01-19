use crate::cli::validate_challenge::validate_challenge;
use crate::cli::validate_proposal::validate_proposal;
use crate::context::ContextPhase;
use crate::models::{Change, ChangePhase, ChallengeVerdict, AgentdConfig, Complexity, ValidationOptions};
use crate::orchestrator::{detect_self_review_marker, find_session_index, GeminiOrchestrator, CodexOrchestrator, SelfReviewResult, UsageMetrics};
use crate::parser::{parse_challenge_verdict, parse_affected_specs};
use crate::orchestrator::prompts;
use crate::state::StateManager;
use crate::Result;
use colored::Colorize;
use std::path::PathBuf;

/// Configuration for the proposal engine
pub struct ProposalEngineConfig {
    pub change_id: String,
    pub description: String,
    pub skip_clarify: bool,
    pub project_root: PathBuf,
    pub config: AgentdConfig,
}

/// Result from running the proposal engine loop
pub struct ProposalEngineResult {
    pub resolved_change_id: String,
    pub verdict: ChallengeVerdict,
    pub iteration_count: usize,
}

/// Main proposal engine loop that orchestrates the full workflow
pub async fn run_proposal_loop(config: ProposalEngineConfig) -> Result<ProposalEngineResult> {
    let ProposalEngineConfig {
        change_id,
        description,
        skip_clarify: _,
        project_root,
        config: agentd_config,
    } = config;

    // Check if Human-in-the-Loop mode is enabled
    if agentd_config.workflow.human_in_loop {
        println!("{}", "🎯 Human-in-the-Loop mode (proposal → specs → tasks with manual iteration)".cyan());

        // Step 1: Generate proposal, specs, and tasks sequentially
        let resolved_change_id = run_proposal_step_sequential(&change_id, &description, &project_root, &agentd_config).await?;

        // Step 2: Validate proposal format
        let format_valid = run_validate_proposal_step(&resolved_change_id, &project_root)?;
        if !format_valid {
            println!("{}", "⚠️  Format validation failed after sequential generation".yellow());
            println!("   Please manually review and fix: agentd/changes/{}/proposal.md", resolved_change_id);
            std::process::exit(1);
        }

        // Step 3: Challenge with Codex
        let verdict = run_challenge_step(&resolved_change_id, &project_root, &agentd_config).await?;

        // Step 4: Validate challenge format
        let _challenge_valid = run_validate_challenge_step(&resolved_change_id, &project_root)?;

        // For HITL mode, we stop after first challenge (no auto-reproposal loop)
        // Skill will use AskUserQuestion to let user decide next action
        return Ok(ProposalEngineResult {
            resolved_change_id,
            verdict,
            iteration_count: 0,
        });
    }

    println!("{}", "🤖 Fully automated mode (auto-reproposal on NEEDS_REVISION)".cyan());

    // Step 1: Generate proposal (resolves change-id conflicts)
    let resolved_change_id = run_proposal_step(&change_id, &description, &project_root, &agentd_config).await?;

    // Step 2: Validate proposal format (local, saves Codex tokens)
    // Loop with Gemini reproposal until format is valid
    let mut format_valid = run_validate_proposal_step(&resolved_change_id, &project_root)?;
    let mut format_iteration = 0;
    let max_format_iterations = agentd_config.workflow.format_iterations;

    while !format_valid && format_iteration < max_format_iterations {
        format_iteration += 1;
        println!();
        println!(
            "{}",
            format!("🔧 Format issues detected - Auto-fixing with Gemini (iteration {})...", format_iteration).yellow()
        );

        // Reproposal with Gemini to fix format
        run_reproposal_step(&resolved_change_id, &project_root, &agentd_config).await?;

        // Re-validate
        println!();
        println!("{}", format!("📋 Re-validating format (iteration {})...", format_iteration).cyan());
        format_valid = run_validate_proposal_step(&resolved_change_id, &project_root)?;
    }

    if !format_valid {
        println!();
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
        println!(
            "{}",
            format!("⚠️  Format validation still failing after {} iterations", max_format_iterations).yellow().bold()
        );
        println!("   Fix manually and re-run: agentd challenge {}", resolved_change_id);
        std::process::exit(1);
    }

    // Step 3: First challenge (use resolved ID)
    let verdict = run_challenge_step(&resolved_change_id, &project_root, &agentd_config).await?;

    // Step 4: Validate challenge format (local)
    let _challenge_valid = run_validate_challenge_step(&resolved_change_id, &project_root)?;

    // Planning iteration loop
    let max_iterations = agentd_config.workflow.planning_iterations;
    let mut current_verdict = verdict;
    let mut iteration: usize = 0;

    loop {
        match current_verdict {
            ChallengeVerdict::Approved => {
                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                if iteration == 0 {
                    println!("{}", "✨ Proposal completed and approved!".green().bold());
                } else {
                    println!("{}", format!("✨ Fixed and approved (iteration {})!", iteration).green().bold());
                }
                println!("   Location: agentd/changes/{}", resolved_change_id);
                println!();

                // Auto-open viewer (if ui feature is enabled)
                open_viewer_if_available(&resolved_change_id, &project_root);

                println!("{}", "⏭️  Next steps:".yellow());
                println!("   agentd implement {}", resolved_change_id);

                return Ok(ProposalEngineResult {
                    resolved_change_id,
                    verdict: ChallengeVerdict::Approved,
                    iteration_count: iteration,
                });
            }
            ChallengeVerdict::NeedsRevision => {
                iteration += 1;
                if iteration > max_iterations as usize {
                    println!();
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                    println!(
                        "{}",
                        format!("⚠️  Automatic refinement limit reached ({} iterations)", max_iterations).yellow().bold()
                    );
                    println!();
                    display_remaining_issues(&resolved_change_id, &project_root)?;
                    std::process::exit(1);
                }

                println!();
                println!(
                    "{}",
                    format!("⚠️  NEEDS_REVISION - Auto-fixing (iteration {})...", iteration).yellow()
                );

                // Reproposal with Gemini
                run_reproposal_step(&resolved_change_id, &project_root, &agentd_config).await?;

                // Re-challenge with Codex
                current_verdict = run_rechallenge_step(&resolved_change_id, &project_root, &agentd_config).await?;
            }
            ChallengeVerdict::Rejected => {
                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                println!("{}", "❌ Proposal rejected".red().bold());
                println!();
                display_remaining_issues(&resolved_change_id, &project_root)?;
                std::process::exit(1);
            }
            ChallengeVerdict::Unknown => {
                println!();
                println!(
                    "{}",
                    "⚠️  Could not parse challenge verdict".yellow()
                );
                println!("   Please review: agentd/changes/{}/CHALLENGE.md", resolved_change_id);
                std::process::exit(1);
            }
        }
    }
}

/// Record LLM usage to StateManager
fn record_usage(
    change_id: &str,
    project_root: &PathBuf,
    step: &str,
    model: &str,
    usage: &UsageMetrics,
    config: &AgentdConfig,
    complexity: Complexity,
) {
    let state_path = project_root
        .join("agentd/changes")
        .join(change_id)
        .join("STATE.yaml");

    if let Ok(mut manager) = StateManager::load(&state_path) {
        // Get pricing from config based on model type
        let (cost_in, cost_out) = if model.contains("gemini") {
            let m = config.gemini.select_model(complexity);
            (m.cost_per_1m_input, m.cost_per_1m_output)
        } else if model.contains("codex") || model.contains("o1") || model.contains("o3") || model.contains("o4-mini") {
            let m = config.codex.select_model(complexity);
            (m.cost_per_1m_input, m.cost_per_1m_output)
        } else {
            let m = config.claude.select_model(complexity);
            (m.cost_per_1m_input, m.cost_per_1m_output)
        };

        manager.record_llm_call(
            step,
            Some(model.to_string()),
            usage.tokens_in,
            usage.tokens_out,
            usage.duration_ms,
            cost_in,
            cost_out,
        );

        let _ = manager.save();
    }
}

/// Step 1: Generate proposal with Gemini
/// Returns the resolved change-id (which may differ from input if conflict occurred)
async fn run_proposal_step(
    change_id: &str,
    description: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<String> {
    println!("{}", "🤖 [1/6] Generating proposal with Gemini...".cyan());

    // Create change directory
    let changes_dir = project_root.join("agentd/changes");
    std::fs::create_dir_all(&changes_dir)?;

    // Resolve change-id conflicts before calling LLMs
    let resolved_change_id = crate::context::resolve_change_id_conflict(change_id, &changes_dir)?;
    let change_dir = changes_dir.join(&resolved_change_id);

    std::fs::create_dir_all(&change_dir)?;

    // Generate GEMINI.md context
    crate::context::generate_gemini_context(&change_dir, ContextPhase::Proposal)?;

    // Create proposal skeleton
    crate::context::create_proposal_skeleton(&change_dir, &resolved_change_id)?;

    // Run Gemini orchestrator with retry
    let orchestrator = GeminiOrchestrator::new(config, project_root);
    let max_retries = config.workflow.script_retries;
    let retry_delay = std::time::Duration::from_secs(config.workflow.retry_delay_secs);

    let mut last_error = None;
    let mut session_id: Option<String> = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            println!(
                "{}",
                format!("🔄 Retrying Gemini proposal (attempt {}/{})", attempt + 1, max_retries + 1).yellow()
            );
            tokio::time::sleep(retry_delay).await;
        }

        // Assess complexity dynamically based on change structure
        let change = Change::new(&resolved_change_id, description);
        let complexity = change.assess_complexity(project_root);

        match orchestrator.run_proposal(&resolved_change_id, description, complexity).await {
            Ok((_output, usage)) => {
                // Save session_id for later use in resume-by-index
                session_id = usage.session_id.clone();

                let model = config.gemini.select_model(complexity).model.clone();
                record_usage(&resolved_change_id, project_root, "proposal", &model, &usage, config, complexity);
                last_error = None;
                break;
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("exit code") || err_msg.contains("connection") || err_msg.contains("timeout") {
                    println!(
                        "{}",
                        format!("⚠️  Gemini proposal failed: {}", err_msg).yellow()
                    );
                    last_error = Some(e);
                } else {
                    return Err(e);
                }
            }
        }
    }

    if let Some(e) = last_error {
        return Err(e);
    }

    // Save session_id to STATE.yaml for resume-by-index
    // R2 requires session_id capture - exit with error if not available
    match &session_id {
        Some(sid) => {
            let state_path = project_root
                .join("agentd/changes")
                .join(&resolved_change_id)
                .join("STATE.yaml");

            if let Ok(mut manager) = StateManager::load(&state_path) {
                manager.set_session_id(sid.clone());
                let _ = manager.save();
            }
        }
        None => {
            println!("{}", "❌ Failed to capture session ID".red().bold());
            std::process::exit(1);
        }
    }

    // Run self-review loop - session_id is guaranteed to exist at this point
    let sid = session_id.as_ref().unwrap();
    println!();
    println!("{}", "🔍 Running self-review...".cyan());

    // Look up session index by UUID - R2 requires exit on failure
    let session_index = match find_session_index(sid, project_root).await {
        Ok(index) => index,
        Err(e) => {
            println!("{}", format!("❌ Session not found, please re-run proposal: {}", e).red().bold());
            std::process::exit(1);
        }
    };

    let change = Change::new(&resolved_change_id, description);
    let complexity = change.assess_complexity(project_root);

    // Self-review loop (max 3 iterations to prevent infinite loops)
    let max_self_review_iterations = 3;
    for iteration in 0..max_self_review_iterations {
        match orchestrator.run_self_review(&resolved_change_id, session_index, complexity).await {
            Ok((output, usage)) => {
                let model = config.gemini.select_model(complexity).model.clone();
                record_usage(&resolved_change_id, project_root, "self-review", &model, &usage, config, complexity);

                let result = detect_self_review_marker(&output);
                match result {
                    SelfReviewResult::Pass => {
                        println!("{}", "Self-review: PASS (no changes)".green());
                        break;
                    }
                    SelfReviewResult::NeedsRevision => {
                        println!("{}", "Self-review: NEEDS_REVISION (files updated)".yellow());
                        if iteration >= max_self_review_iterations - 1 {
                            println!("{}", "⚠️  Self-review still finding issues after max iterations".yellow());
                        }
                        // Continue to next iteration - Gemini already made the fixes
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("⚠️  Self-review failed: {}", e).yellow());
                break;
            }
        }
    }

    // Create Change object
    let mut change = Change::new(&resolved_change_id, description);
    change.update_phase(ChangePhase::Proposed);

    // Validate structure
    match change.validate_structure(project_root) {
        Ok(_) => {
            println!(
                "{}",
                "✅ Proposal generated (proposal.md, tasks.md, specs/)".green()
            );
            Ok(resolved_change_id)
        }
        Err(e) => {
            println!("{}", "⚠️  Warning: Proposal structure incomplete".yellow());
            println!("   {}", e);
            Ok(resolved_change_id)
        }
    }
}

/// Step 2: Validate proposal format (local validation, no AI)
fn run_validate_proposal_step(
    change_id: &str,
    project_root: &PathBuf,
) -> Result<bool> {
    println!();
    println!("{}", "📋 [2/6] Validating proposal format...".cyan());

    let options = ValidationOptions::default();
    let summary = validate_proposal(change_id, project_root, &options)?;

    if summary.is_valid() {
        println!("{}", "✅ Proposal format validation passed".green());
        Ok(true)
    } else {
        println!(
            "{}",
            format!("⚠️  {} HIGH severity format errors found", summary.high_count).yellow()
        );
        // Continue anyway - let Codex find more issues
        Ok(false)
    }
}

/// Step 3: Run challenge with Codex
async fn run_challenge_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<ChallengeVerdict> {
    println!();
    println!("{}", "🔍 [3/6] Challenging proposal with Codex...".cyan());

    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Create Change object and validate
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Generate AGENTS.md context
    crate::context::generate_agents_context(&change_dir, ContextPhase::Challenge)?;

    // Create CHALLENGE.md skeleton
    crate::context::create_challenge_skeleton(&change_dir, change_id)?;

    // Assess complexity dynamically based on change structure
    let change = Change::new(change_id, "");
    let complexity = change.assess_complexity(project_root);

    // Run Codex orchestrator with retry
    let orchestrator = CodexOrchestrator::new(config, project_root);
    let max_retries = config.workflow.script_retries;
    let retry_delay = std::time::Duration::from_secs(config.workflow.retry_delay_secs);

    let mut last_error = None;
    for attempt in 0..=max_retries {
        if attempt > 0 {
            println!(
                "{}",
                format!("🔄 Retrying Codex challenge (attempt {}/{})", attempt + 1, max_retries + 1).yellow()
            );
            tokio::time::sleep(retry_delay).await;
        }

        match orchestrator.run_challenge(change_id, complexity).await {
            Ok((_output, usage)) => {
                let model = config.codex.select_model(complexity).model.clone();
                record_usage(change_id, project_root, "challenge", &model, &usage, config, complexity);
                last_error = None;
                break;
            }
            Err(e) => {
                let err_msg = e.to_string();
                // Check if it's a transient error (connection, timeout, etc.)
                if err_msg.contains("exit code") || err_msg.contains("connection") || err_msg.contains("timeout") {
                    println!(
                        "{}",
                        format!("⚠️  Codex challenge failed: {}", err_msg).yellow()
                    );
                    last_error = Some(e);
                } else {
                    // Non-transient error, don't retry
                    return Err(e);
                }
            }
        }
    }

    if let Some(e) = last_error {
        return Err(e);
    }

    // Parse verdict
    let challenge_path = change.challenge_path(project_root);
    if !challenge_path.exists() {
        println!("{}", "⚠️  CHALLENGE.md not created".yellow());
        return Ok(ChallengeVerdict::Unknown);
    }

    let verdict = parse_challenge_verdict(&challenge_path)?;

    // Display summary
    let content = std::fs::read_to_string(&challenge_path)?;
    display_challenge_summary(&content, &verdict);

    Ok(verdict)
}

/// Step 4: Validate challenge format (local validation, no AI)
fn run_validate_challenge_step(
    change_id: &str,
    project_root: &PathBuf,
) -> Result<bool> {
    println!();
    println!("{}", "📋 [4/6] Validating challenge format...".cyan());

    let options = ValidationOptions::default();
    let result = validate_challenge(change_id, project_root, &options)?;

    if result.is_valid() {
        println!("{}", "✅ Challenge format validation passed".green());
        Ok(true)
    } else {
        println!(
            "{}",
            format!("⚠️  Challenge format issues: {:?}", result.errors).yellow()
        );
        Ok(false)
    }
}

/// Step 6: Run re-challenge with Codex (resumes session for cached context)
async fn run_rechallenge_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<ChallengeVerdict> {
    println!();
    println!("{}", "🔍 [6/6] Re-challenging with Codex (cached session)...".cyan());

    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Create Change object and validate
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Assess complexity dynamically based on change structure
    let complexity = change.assess_complexity(project_root);

    // Regenerate AGENTS.md context
    crate::context::generate_agents_context(&change_dir, ContextPhase::Challenge)?;

    // Recreate CHALLENGE.md skeleton for re-challenge
    crate::context::create_challenge_skeleton(&change_dir, change_id)?;

    // Run Codex rechallenge orchestrator (resumes session)
    let orchestrator = CodexOrchestrator::new(config, project_root);
    let (_output, usage) = orchestrator.run_rechallenge(change_id, complexity).await?;
    let model = config.codex.select_model(complexity).model.clone();
    record_usage(change_id, project_root, "rechallenge", &model, &usage, config, complexity);

    // Parse verdict
    let challenge_path = change.challenge_path(project_root);
    if !challenge_path.exists() {
        println!("{}", "⚠️  CHALLENGE.md not created".yellow());
        return Ok(ChallengeVerdict::Unknown);
    }

    let verdict = parse_challenge_verdict(&challenge_path)?;

    // Display summary
    let content = std::fs::read_to_string(&challenge_path)?;
    display_challenge_summary(&content, &verdict);

    Ok(verdict)
}

/// Step 5: Run reproposal with Gemini (resumes session for cached context)
async fn run_reproposal_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<()> {
    println!();
    println!("{}", "🔄 [5/6] Auto-fixing with Gemini reproposal...".cyan());

    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Assess complexity dynamically based on change structure
    let change = Change::new(change_id, "");
    let complexity = change.assess_complexity(project_root);

    // Regenerate GEMINI.md context
    crate::context::generate_gemini_context(&change_dir, ContextPhase::Proposal)?;

    // Run Gemini reproposal orchestrator with retry
    let orchestrator = GeminiOrchestrator::new(config, project_root);
    let max_retries = config.workflow.script_retries;
    let retry_delay = std::time::Duration::from_secs(config.workflow.retry_delay_secs);

    // Load session_id from STATE.yaml for resume-by-index - R2 requires strict enforcement
    let state_path = project_root
        .join("agentd/changes")
        .join(change_id)
        .join("STATE.yaml");

    let session_index = match StateManager::load(&state_path) {
        Ok(manager) => {
            match manager.session_id() {
                Some(sid) => {
                    match find_session_index(sid, project_root).await {
                        Ok(index) => index,
                        Err(e) => {
                            println!("{}", format!("❌ Session not found, please re-run proposal: {}", e).red().bold());
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    println!("{}", "❌ Failed to capture session ID".red().bold());
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            println!("{}", "❌ Failed to load STATE.yaml".red().bold());
            std::process::exit(1);
        }
    };

    let mut last_error = None;
    for attempt in 0..=max_retries {
        if attempt > 0 {
            println!(
                "{}",
                format!("🔄 Retrying Gemini reproposal (attempt {}/{})", attempt + 1, max_retries + 1).yellow()
            );
            tokio::time::sleep(retry_delay).await;
        }

        // Use resume-by-index (R2 requires strict enforcement - no fallback to --resume latest)
        let result = orchestrator.run_reproposal_with_session(change_id, session_index, complexity).await;

        match result {
            Ok((_output, usage)) => {
                let model = config.gemini.select_model(complexity).model.clone();
                record_usage(change_id, project_root, "reproposal", &model, &usage, config, complexity);
                last_error = None;
                break;
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("exit code") || err_msg.contains("connection") || err_msg.contains("timeout") {
                    println!(
                        "{}",
                        format!("⚠️  Gemini reproposal failed: {}", err_msg).yellow()
                    );
                    last_error = Some(e);
                } else {
                    return Err(e);
                }
            }
        }
    }

    if let Some(e) = last_error {
        return Err(e);
    }

    println!("{}", "✅ Proposal updated based on challenge feedback".green());

    Ok(())
}

/// Display challenge summary
fn display_challenge_summary(content: &str, verdict: &ChallengeVerdict) {
    let high_count = content.matches("**Severity**: High").count();
    let medium_count = content.matches("**Severity**: Medium").count();
    let low_count = content.matches("**Severity**: Low").count();

    match verdict {
        ChallengeVerdict::Approved => {
            println!("{}", "✅ APPROVED - Ready for implementation!".green().bold());
        }
        ChallengeVerdict::NeedsRevision => {
            print!("{}", "⚠️  NEEDS_REVISION".yellow().bold());
            if high_count > 0 || medium_count > 0 {
                print!(" - Found ");
                if high_count > 0 {
                    print!("{} HIGH", high_count);
                }
                if high_count > 0 && medium_count > 0 {
                    print!(", ");
                }
                if medium_count > 0 {
                    print!("{} MEDIUM", medium_count);
                }
                println!(" severity issues");
            } else {
                println!();
            }
        }
        ChallengeVerdict::Rejected => {
            println!("{}", "❌ REJECTED - Fundamental problems".red().bold());
        }
        ChallengeVerdict::Unknown => {
            println!("{}", "❓ UNKNOWN - Could not parse verdict".yellow());
        }
    }

    if high_count > 0 || medium_count > 0 || low_count > 0 {
        println!("   {} HIGH, {} MEDIUM, {} LOW severity issues",
            high_count, medium_count, low_count);
    }
}

/// Display remaining issues after auto-fix failed
fn display_remaining_issues(change_id: &str, project_root: &PathBuf) -> Result<()> {
    let challenge_path = project_root
        .join("agentd/changes")
        .join(change_id)
        .join("CHALLENGE.md");

    if !challenge_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&challenge_path)?;
    let high_count = content.matches("**Severity**: High").count();

    println!("   The following issues could not be auto-fixed:");
    println!();

    // Try to extract first HIGH severity issue as example
    if high_count > 0 {
        if let Some(issue_start) = content.find("**Severity**: High") {
            let section = &content[issue_start.saturating_sub(200)
                ..issue_start.saturating_add(300).min(content.len())];

            println!("   {}", "HIGH Severity (example):".red().bold());

            // Try to extract description
            if let Some(desc_start) = section.find("**Description**:") {
                if let Some(desc_end) = section[desc_start..].find('\n') {
                    let desc = &section[desc_start + 16..desc_start + desc_end];
                    println!("   • {}", desc.trim());
                }
            }
            println!();
        }
    }

    println!("   Please manually review and edit:");
    println!("     agentd/changes/{}/proposal.md", change_id);
    println!("     agentd/changes/{}/CHALLENGE.md (full report)", change_id);
    println!();
    println!("   Then run:");
    println!("     agentd challenge {}", change_id);
    println!("     agentd reproposal {}  (if needed)", change_id);

    Ok(())
}

/// Open the plan viewer if the ui feature is enabled
/// Spawns a detached process so the CLI can exit independently
#[cfg(feature = "ui")]
fn open_viewer_if_available(change_id: &str, _project_root: &PathBuf) {
    println!("{}", "🖼️  Opening plan viewer...".cyan());
    match crate::cli::view::spawn_detached(change_id) {
        Ok(_) => {
            println!("   Plan viewer opened in a new window.");
        }
        Err(e) => {
            println!("{}", format!("   Warning: Could not open viewer: {}", e).yellow());
            println!("   You can manually open it with: agentd view {}", change_id);
        }
    }
    println!();
}

#[cfg(not(feature = "ui"))]
fn open_viewer_if_available(change_id: &str, project_root: &PathBuf) {
    let change_path = project_root.join("agentd/changes").join(change_id);
    // Print exact message without ANSI formatting to match spec requirement
    println!("UI feature disabled. View plan manually at: {}", change_path.display());
    println!();
}

/// Sequential generation workflow: proposal → specs → tasks
/// Each phase uses fresh session, context passed via reviewed files (MCP read_file)
pub async fn run_proposal_step_sequential(
    change_id: &str,
    description: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<String> {
    println!("{}", "🎯 Multi-Phase Sequential Generation".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());

    // Create change directory
    let changes_dir = project_root.join("agentd/changes");
    std::fs::create_dir_all(&changes_dir)?;

    // Resolve change-id conflicts
    let resolved_change_id = crate::context::resolve_change_id_conflict(change_id, &changes_dir)?;
    let change_dir = changes_dir.join(&resolved_change_id);
    std::fs::create_dir_all(&change_dir)?;

    // Assess complexity
    let change = Change::new(&resolved_change_id, description);
    let complexity = change.assess_complexity(project_root);

    let orchestrator = GeminiOrchestrator::new(config, project_root);

    // ====================
    // Phase 1: Generate proposal.md
    // ====================
    println!();
    println!("{}", "📝 Phase 1/3: Generating proposal.md...".cyan().bold());

    // Generate GEMINI.md context for proposal phase
    crate::context::generate_gemini_context(&change_dir, ContextPhase::Proposal)?;

    let proposal_prompt = prompts::gemini_proposal_with_mcp_prompt(&resolved_change_id, description);

    // Run one-shot generation (fresh session)
    let (_output, usage) = orchestrator.run_one_shot(&resolved_change_id, &proposal_prompt, complexity).await?;
    let model = config.gemini.select_model(complexity).model.clone();
    record_usage(&resolved_change_id, project_root, "proposal-gen", &model, &usage, config, complexity);

    println!("{}", "✅ proposal.md generated".green());

    // Self-review loop for proposal (max 3 iterations, each with fresh session)
    println!("{}", "🔍 Self-reviewing proposal.md...".cyan());
    let max_review_iterations = 3;

    for iteration in 0..max_review_iterations {
        let review_prompt = prompts::proposal_self_review_with_mcp_prompt(&resolved_change_id);

        match orchestrator.run_one_shot(&resolved_change_id, &review_prompt, complexity).await {
            Ok((review_output, review_usage)) => {
                let model = config.gemini.select_model(complexity).model.clone();
                record_usage(&resolved_change_id, project_root, "proposal-review", &model, &review_usage, config, complexity);

                let result = detect_self_review_marker(&review_output);
                match result {
                    SelfReviewResult::Pass => {
                        println!("{}", format!("   ✓ Review {}: PASS", iteration + 1).green());
                        break;
                    }
                    SelfReviewResult::NeedsRevision => {
                        println!("{}", format!("   ⚠ Review {}: NEEDS_REVISION (auto-fixed)", iteration + 1).yellow());
                        if iteration >= max_review_iterations - 1 {
                            println!("{}", "   ⚠ Max review iterations reached".yellow());
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("   ⚠ Review {} failed: {}", iteration + 1, e).yellow());
                break;
            }
        }
    }

    // Parse affected_specs from proposal.md
    let proposal_path = change_dir.join("proposal.md");
    let proposal_content = std::fs::read_to_string(&proposal_path)?;
    let affected_specs = parse_affected_specs(&proposal_content)?;

    if affected_specs.is_empty() {
        println!("{}", "ℹ️  No specs required for this change".blue());
    } else {
        println!("{}", format!("📋 Found {} specs to generate: {:?}", affected_specs.len(), affected_specs).cyan());
    }

    // ====================
    // Phase 2: Generate specs sequentially
    // ====================
    if !affected_specs.is_empty() {
        println!();
        println!("{}", format!("📝 Phase 2/3: Generating {} specs...", affected_specs.len()).cyan().bold());

        for (idx, spec_id) in affected_specs.iter().enumerate() {
            println!();
            println!("{}", format!("  📄 Spec {}/{}: {}", idx + 1, affected_specs.len(), spec_id).cyan());

            // Prepare context: proposal.md + previously generated specs
            let mut context_specs = vec![];
            for prev_spec_id in &affected_specs[..idx] {
                context_specs.push(prev_spec_id.clone());
            }

            let spec_prompt = prompts::gemini_spec_with_mcp_prompt(
                &resolved_change_id,
                spec_id,
                &context_specs,
            );

            // Run one-shot generation (fresh session)
            let (_spec_output, spec_usage) = orchestrator.run_one_shot(&resolved_change_id, &spec_prompt, complexity).await?;
            let model = config.gemini.select_model(complexity).model.clone();
            record_usage(&resolved_change_id, project_root, &format!("spec-gen-{}", spec_id), &model, &spec_usage, config, complexity);

            println!("{}", format!("     ✅ {}.md generated", spec_id).green());

            // Self-review loop for this spec
            println!("{}", format!("     🔍 Self-reviewing {}...", spec_id).cyan());

            for review_iter in 0..max_review_iterations {
                let spec_review_prompt = prompts::spec_self_review_with_mcp_prompt(&resolved_change_id, spec_id, &context_specs);

                match orchestrator.run_one_shot(&resolved_change_id, &spec_review_prompt, complexity).await {
                    Ok((spec_review_output, spec_review_usage)) => {
                        let model = config.gemini.select_model(complexity).model.clone();
                        record_usage(&resolved_change_id, project_root, &format!("spec-review-{}", spec_id), &model, &spec_review_usage, config, complexity);

                        let result = detect_self_review_marker(&spec_review_output);
                        match result {
                            SelfReviewResult::Pass => {
                                println!("{}", format!("        ✓ Review {}: PASS", review_iter + 1).green());
                                break;
                            }
                            SelfReviewResult::NeedsRevision => {
                                println!("{}", format!("        ⚠ Review {}: NEEDS_REVISION (auto-fixed)", review_iter + 1).yellow());
                                if review_iter >= max_review_iterations - 1 {
                                    println!("{}", "        ⚠ Max review iterations reached".yellow());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("{}", format!("        ⚠ Review {} failed: {}", review_iter + 1, e).yellow());
                        break;
                    }
                }
            }
        }
    }

    // ====================
    // Phase 3: Generate tasks.md
    // ====================
    println!();
    println!("{}", "📝 Phase 3/3: Generating tasks.md...".cyan().bold());

    let tasks_prompt = prompts::gemini_tasks_with_mcp_prompt(&resolved_change_id, &affected_specs);

    // Run one-shot generation (fresh session)
    let (_tasks_output, tasks_usage) = orchestrator.run_one_shot(&resolved_change_id, &tasks_prompt, complexity).await?;
    let model = config.gemini.select_model(complexity).model.clone();
    record_usage(&resolved_change_id, project_root, "tasks-gen", &model, &tasks_usage, config, complexity);

    println!("{}", "✅ tasks.md generated".green());

    // Self-review loop for tasks
    println!("{}", "🔍 Self-reviewing tasks.md...".cyan());

    // Prepare all files for tasks review (proposal.md + all specs)
    let mut all_files = vec!["proposal.md".to_string()];
    for spec_id in &affected_specs {
        all_files.push(format!("specs/{}.md", spec_id));
    }

    for iteration in 0..max_review_iterations {
        let tasks_review_prompt = prompts::tasks_self_review_with_mcp_prompt(&resolved_change_id, &all_files);

        match orchestrator.run_one_shot(&resolved_change_id, &tasks_review_prompt, complexity).await {
            Ok((tasks_review_output, tasks_review_usage)) => {
                let model = config.gemini.select_model(complexity).model.clone();
                record_usage(&resolved_change_id, project_root, "tasks-review", &model, &tasks_review_usage, config, complexity);

                let result = detect_self_review_marker(&tasks_review_output);
                match result {
                    SelfReviewResult::Pass => {
                        println!("{}", format!("   ✓ Review {}: PASS", iteration + 1).green());
                        break;
                    }
                    SelfReviewResult::NeedsRevision => {
                        println!("{}", format!("   ⚠ Review {}: NEEDS_REVISION (auto-fixed)", iteration + 1).yellow());
                        if iteration >= max_review_iterations - 1 {
                            println!("{}", "   ⚠ Max review iterations reached".yellow());
                        }
                    }
                }
            }
            Err(e) => {
                println!("{}", format!("   ⚠ Review {} failed: {}", iteration + 1, e).yellow());
                break;
            }
        }
    }

    // ====================
    // Finalization
    // ====================
    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!("{}", "✨ Sequential generation completed!".green().bold());
    println!("{}", format!("   Location: agentd/changes/{}", resolved_change_id).cyan());

    // Update change phase
    let mut change = Change::new(&resolved_change_id, description);
    change.update_phase(ChangePhase::Proposed);

    // Validate structure
    match change.validate_structure(project_root) {
        Ok(_) => {
            println!("{}", "✅ All files validated".green());
            Ok(resolved_change_id)
        }
        Err(e) => {
            println!("{}", format!("⚠️  Warning: Structure validation issues: {}", e).yellow());
            Ok(resolved_change_id)
        }
    }
}
