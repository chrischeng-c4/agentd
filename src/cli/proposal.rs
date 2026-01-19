use crate::cli::proposal_engine;
use crate::models::AgentdConfig;
use crate::Result;
use colored::Colorize;
use std::env;

pub struct ProposalCommand;

/// Main entry point for the proposal workflow with automatic challenge-reproposal loop
///
/// Workflow (6 steps):
/// 0. Check clarifications.md (unless skip_clarify)
/// 1. Generate proposal (Gemini)
/// 2. Validate proposal format (local)
/// 3. Challenge proposal (Codex)
/// 4. Validate challenge (local)
/// 5. Reproposal (Gemini) - if NEEDS_REVISION
/// 6. Re-challenge (Codex) - one loop only
pub async fn run(change_id: &str, description: &str, skip_clarify: bool) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;

    println!(
        "{}",
        "🎭 Agentd Proposal Workflow".cyan().bold()
    );
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    // Step 0: Check for clarifications.md (unless skipped)
    if !skip_clarify {
        let clarifications_path = project_root
            .join("agentd/changes")
            .join(change_id)
            .join("clarifications.md");

        if !clarifications_path.exists() {
            println!("{}", "❌ Error: clarifications.md not found".red().bold());
            println!();
            println!("   Run clarification first via /agentd:plan, or use --skip-clarify to bypass.");
            println!();
            println!("   Example:");
            println!("     agentd proposal {} \"{}\" --skip-clarify", change_id, description);
            return Ok(());
        }
        println!("{}", "✅ clarifications.md found".green());
    } else {
        println!("{}", "⏭️  Skipping clarification check (--skip-clarify)".yellow());
    }

    // Delegate to shared engine
    let engine_config = proposal_engine::ProposalEngineConfig {
        change_id: change_id.to_string(),
        description: description.to_string(),
        skip_clarify,
        project_root,
        config,
    };

    proposal_engine::run_proposal_loop(engine_config).await?;
    Ok(())
}
