use crate::models::{Change, ChangePhase, ChallengeVerdict, AgentdConfig};
use crate::orchestrator::ScriptRunner;
use crate::parser::parse_challenge_verdict;
use crate::Result;
use colored::Colorize;
use std::env;
use std::path::PathBuf;

pub struct ProposalCommand;

/// Main entry point for the proposal workflow with automatic challenge-reproposal loop
pub async fn run(change_id: &str, description: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;

    println!(
        "{}",
        "🎭 Agentd Proposal Workflow".cyan().bold()
    );
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    // Step 1: Generate proposal (resolves change-id conflicts)
    let resolved_change_id = run_proposal_step(change_id, description, &project_root, &config).await?;

    // Step 2: First challenge (use resolved ID)
    let verdict = run_challenge_step(&resolved_change_id, &project_root, &config).await?;

    match verdict {
        ChallengeVerdict::Approved => {
            println!();
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
            println!("{}", "✨ Proposal completed and approved!".green().bold());
            println!("   Location: agentd/changes/{}", resolved_change_id);
            println!();
            println!("{}", "⏭️  Next steps:".yellow());
            println!("   agentd implement {}", resolved_change_id);
            return Ok(());
        }
        ChallengeVerdict::NeedsRevision => {
            println!();
            println!(
                "{}",
                "⚠️  NEEDS_REVISION - Auto-fixing with Gemini...".yellow()
            );

            // Step 3: Auto reproposal (one time only, resumes Gemini session)
            run_reproposal_step(&resolved_change_id, &project_root, &config).await?;

            // Step 4: Second challenge (resumes Codex session)
            let verdict2 = run_rechallenge_step(&resolved_change_id, &project_root, &config).await?;

            match verdict2 {
                ChallengeVerdict::Approved => {
                    println!();
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                    println!("{}", "✨ Fixed and approved!".green().bold());
                    println!("   Location: agentd/changes/{}", resolved_change_id);
                    println!();
                    println!("{}", "⏭️  Next steps:".yellow());
                    println!("   agentd implement {}", resolved_change_id);
                    Ok(())
                }
                ChallengeVerdict::NeedsRevision => {
                    println!();
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                    println!(
                        "{}",
                        "⚠️  Automatic refinement limit reached (1 iteration)".yellow().bold()
                    );
                    println!();
                    display_remaining_issues(&resolved_change_id, &project_root)?;
                    Ok(())
                }
                ChallengeVerdict::Rejected => {
                    println!();
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                    println!("{}", "❌ Proposal rejected".red().bold());
                    println!();
                    display_remaining_issues(&resolved_change_id, &project_root)?;
                    Ok(())
                }
                ChallengeVerdict::Unknown => {
                    println!();
                    println!(
                        "{}",
                        "⚠️  Could not parse challenge verdict".yellow()
                    );
                    println!("   Please review: agentd/changes/{}/CHALLENGE.md", resolved_change_id);
                    Ok(())
                }
            }
        }
        ChallengeVerdict::Rejected => {
            println!();
            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
            println!("{}", "❌ Proposal rejected".red().bold());
            println!();
            display_remaining_issues(&resolved_change_id, &project_root)?;
            Ok(())
        }
        ChallengeVerdict::Unknown => {
            println!();
            println!(
                "{}",
                "⚠️  Could not parse challenge verdict".yellow()
            );
            println!("   Please review: agentd/changes/{}/CHALLENGE.md", resolved_change_id);
            Ok(())
        }
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
    println!("{}", "🤖 [1/4] Generating proposal with Gemini...".cyan());

    // Create change directory
    let changes_dir = project_root.join("agentd/changes");
    std::fs::create_dir_all(&changes_dir)?;

    // Resolve change-id conflicts before calling LLMs
    let resolved_change_id = crate::context::resolve_change_id_conflict(change_id, &changes_dir)?;
    let change_dir = changes_dir.join(&resolved_change_id);

    std::fs::create_dir_all(&change_dir)?;

    // Generate GEMINI.md context
    crate::context::generate_gemini_context(&change_dir)?;

    // Create proposal skeleton
    crate::context::create_proposal_skeleton(&change_dir, &resolved_change_id)?;

    // Run Gemini script
    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    let _output = script_runner
        .run_gemini_proposal(&resolved_change_id, description)
        .await?;

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

/// Step 2/4: Run challenge with Codex
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
    crate::context::generate_agents_context(&change_dir)?;

    // Create CHALLENGE.md skeleton
    crate::context::create_challenge_skeleton(&change_dir, change_id)?;

    // Run Codex script
    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    let _output = script_runner.run_codex_challenge(change_id).await?;

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

/// Step 4: Run re-challenge with Codex (resumes session for cached context)
async fn run_rechallenge_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<ChallengeVerdict> {
    println!();
    println!("{}", "🔍 [4/4] Re-challenging with Codex (cached session)...".cyan());

    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Create Change object and validate
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Regenerate AGENTS.md context
    crate::context::generate_agents_context(&change_dir)?;

    // Recreate CHALLENGE.md skeleton for re-challenge
    crate::context::create_challenge_skeleton(&change_dir, change_id)?;

    // Run Codex rechallenge script (resumes session)
    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    let _output = script_runner.run_codex_rechallenge(change_id).await?;

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

/// Step 3: Run reproposal with Gemini (resumes session for cached context)
async fn run_reproposal_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<()> {
    println!();
    println!("{}", "🔄 [3/4] Auto-fixing with Gemini reproposal...".cyan());

    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Regenerate GEMINI.md context
    crate::context::generate_gemini_context(&change_dir)?;

    // Run Gemini reproposal
    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    let _output = script_runner.run_gemini_reproposal(change_id).await?;

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
