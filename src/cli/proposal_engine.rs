use crate::context::ContextPhase;
use crate::models::{Change, ChangePhase, ChallengeVerdict, AgentdConfig, Complexity};
use crate::orchestrator::{detect_self_review_marker, GeminiOrchestrator, CodexOrchestrator, SelfReviewResult, UsageMetrics};
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
    /// True if only LOW severity issues remain (or at most 1 MEDIUM)
    pub has_only_minor_issues: bool,
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

    println!("{}", "🎯 Sequential MCP Generation (proposal → specs → tasks)".cyan());

    // Step 1: Generate proposal, specs, and tasks sequentially
    let resolved_change_id = run_proposal_step_sequential(&change_id, &description, &project_root, &agentd_config).await?;

    // Step 2: Challenge with Codex
    let mut verdict = run_challenge_step(&resolved_change_id, &project_root, &agentd_config).await?;

    // Step 3: Reproposal loop (up to planning_iterations times)
    let max_iterations = agentd_config.workflow.planning_iterations;
    let mut iteration: usize = 0;

    while verdict == ChallengeVerdict::NeedsRevision && iteration < max_iterations as usize {
        iteration += 1;
        println!();
        println!(
            "{}",
            format!("🔄 NEEDS_REVISION - Auto-fixing with reproposal (iteration {}/{})...", iteration, max_iterations).yellow()
        );

        // Reproposal with Gemini
        run_reproposal_step(&resolved_change_id, &project_root, &agentd_config).await?;

        // Re-challenge with Codex
        verdict = run_rechallenge_step(&resolved_change_id, &project_root, &agentd_config).await?;
    }

    // Get proposal path for issue severity check
    let proposal_path = project_root.join("agentd/changes").join(&resolved_change_id).join("proposal.md");
    let has_only_minor_issues = check_only_minor_issues(&proposal_path).unwrap_or(false);

    // Display final result
    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());

    match &verdict {
        ChallengeVerdict::Approved => {
            if iteration == 0 {
                println!("{}", "✨ Proposal approved!".green().bold());
            } else {
                println!("{}", format!("✨ Fixed and approved (after {} iterations)!", iteration).green().bold());
            }
            println!("   Location: agentd/changes/{}", resolved_change_id);
        }
        ChallengeVerdict::NeedsRevision => {
            if iteration >= max_iterations as usize {
                println!(
                    "{}",
                    format!("⚠️  Reached iteration limit ({} iterations)", max_iterations).yellow().bold()
                );
            } else {
                println!("{}", "⚠️  NEEDS_REVISION".yellow().bold());
            }
            if has_only_minor_issues {
                println!("   Only minor issues remain - can proceed to implementation.");
            } else {
                println!("   Review the remaining issues in proposal.md");
            }
            println!("   Location: agentd/changes/{}", resolved_change_id);
        }
        ChallengeVerdict::Rejected => {
            println!("{}", "❌ Proposal rejected".red().bold());
            display_remaining_issues(&resolved_change_id, &project_root)?;
        }
        ChallengeVerdict::Unknown => {
            println!("{}", "❓ Could not parse challenge verdict".yellow());
            println!("   Please review: agentd/changes/{}/proposal.md", resolved_change_id);
        }
    }

    // Return result - Skill will use AskUserQuestion for next action
    Ok(ProposalEngineResult {
        resolved_change_id,
        verdict,
        iteration_count: iteration,
        has_only_minor_issues,
    })
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

/// Step 2: Run challenge with Codex
async fn run_challenge_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<ChallengeVerdict> {
    println!();
    println!("{}", "🔍 [2/4] Challenging proposal with Codex...".cyan());

    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Create Change object and validate
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Generate AGENTS.md context
    crate::context::generate_agents_context(&change_dir, ContextPhase::Challenge)?;

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

    // Parse verdict from proposal.md review block
    let proposal_path = change.proposal_path(project_root);
    let verdict = parse_challenge_verdict(&proposal_path)?;

    if verdict == ChallengeVerdict::Unknown {
        println!("{}", "⚠️  No review block found in proposal.md".yellow());
        return Ok(ChallengeVerdict::Unknown);
    }

    // Display summary from review block
    let content = std::fs::read_to_string(&proposal_path)?;
    if let Ok(Some(review)) = crate::parser::parse_latest_review(&content) {
        display_challenge_summary(&review.content, &verdict);
    }

    Ok(verdict)
}

/// Step 3: Run re-challenge with Codex (fresh session with existing documents as context)
async fn run_rechallenge_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<ChallengeVerdict> {
    println!();
    println!("{}", "🔍 [4/4] Re-challenging with Codex...".cyan());

    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Create Change object and validate
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Assess complexity dynamically based on change structure
    let complexity = change.assess_complexity(project_root);

    // Regenerate AGENTS.md context
    crate::context::generate_agents_context(&change_dir, ContextPhase::Challenge)?;

    // Run Codex rechallenge orchestrator (fresh session - AGENTS.md provides context)
    let orchestrator = CodexOrchestrator::new(config, project_root);
    let (_output, usage) = orchestrator.run_rechallenge_fresh(change_id, complexity).await?;
    let model = config.codex.select_model(complexity).model.clone();
    record_usage(change_id, project_root, "rechallenge", &model, &usage, config, complexity);

    // Parse verdict from proposal.md review block
    let proposal_path = change.proposal_path(project_root);
    let verdict = parse_challenge_verdict(&proposal_path)?;

    if verdict == ChallengeVerdict::Unknown {
        println!("{}", "⚠️  No review block found in proposal.md".yellow());
        return Ok(ChallengeVerdict::Unknown);
    }

    // Display summary from review block
    let content = std::fs::read_to_string(&proposal_path)?;
    if let Ok(Some(review)) = crate::parser::parse_latest_review(&content) {
        display_challenge_summary(&review.content, &verdict);
    }

    Ok(verdict)
}

/// Step 3 (loop): Run reproposal with Gemini (fresh session with existing documents as context)
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

    // Regenerate GEMINI.md context (includes existing proposal, specs, and challenge feedback)
    crate::context::generate_gemini_context(&change_dir, ContextPhase::Proposal)?;

    // Run Gemini reproposal orchestrator with retry (fresh session - documents provide context)
    let orchestrator = GeminiOrchestrator::new(config, project_root);
    let max_retries = config.workflow.script_retries;
    let retry_delay = std::time::Duration::from_secs(config.workflow.retry_delay_secs);

    let mut last_error = None;
    for attempt in 0..=max_retries {
        if attempt > 0 {
            println!(
                "{}",
                format!("🔄 Retrying Gemini reproposal (attempt {}/{})", attempt + 1, max_retries + 1).yellow()
            );
            tokio::time::sleep(retry_delay).await;
        }

        // Fresh session - GEMINI.md contains all necessary context (proposal, specs, review feedback)
        let result = orchestrator.run_reproposal_fresh(change_id, complexity).await;

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
    let proposal_path = project_root
        .join("agentd/changes")
        .join(change_id)
        .join("proposal.md");

    if !proposal_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&proposal_path)?;

    // Try to extract review content
    let review_content = if let Ok(Some(review)) = crate::parser::parse_latest_review(&content) {
        review.content
    } else {
        return Ok(());
    };

    let high_count = review_content.matches("**Severity**: High").count();

    println!("   The following issues could not be auto-fixed:");
    println!();

    // Try to extract first HIGH severity issue as example
    if high_count > 0 {
        if let Some(issue_start) = review_content.find("**Severity**: High") {
            let section = &review_content[issue_start.saturating_sub(200)
                ..issue_start.saturating_add(300).min(review_content.len())];

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
    println!();
    println!("   Then run:");
    println!("     agentd challenge {}", change_id);
    println!("     agentd reproposal {}  (if needed)", change_id);

    Ok(())
}

/// Check if only minor issues remain in the proposal
/// Returns true if no HIGH severity issues and at most 1 MEDIUM severity issue
fn check_only_minor_issues(proposal_path: &std::path::Path) -> Result<bool> {
    if !proposal_path.exists() {
        return Ok(true); // No proposal means no issues
    }

    let content = std::fs::read_to_string(proposal_path)?;

    // Try to extract review content
    let review_content = if let Ok(Some(review)) = crate::parser::parse_latest_review(&content) {
        review.content
    } else {
        return Ok(true); // No review means no issues
    };

    let high_count = review_content.matches("**Severity**: High").count();
    let medium_count = review_content.matches("**Severity**: Medium").count();

    // Only minor if no HIGH and at most 1 MEDIUM
    Ok(high_count == 0 && medium_count <= 1)
}

/// Open the plan viewer if the ui feature is enabled
/// Spawns a detached process so the CLI can exit independently
#[allow(dead_code)]
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

#[allow(dead_code)]
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

    // Self-review loop for proposal (max 1 iteration - Codex challenge catches remaining issues)
    println!("{}", "🔍 Self-reviewing proposal.md...".cyan());
    let max_review_iterations = 1;

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
