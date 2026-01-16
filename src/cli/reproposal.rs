use crate::context::ContextPhase;
use crate::orchestrator::GeminiOrchestrator;
use crate::{
    models::{Change, AgentdConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub struct ReproposalCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;

    // Check if change and challenge exist
    let change = Change::new(change_id, "");
    let challenge_path = change.challenge_path(&project_root);

    if !challenge_path.exists() {
        anyhow::bail!(
            "No challenge found for '{}'. Run 'agentd challenge {}' first.",
            change_id,
            change_id
        );
    }

    // Assess complexity dynamically based on change structure
    let change = Change::new(change_id, "");
    let complexity = change.assess_complexity(&project_root);

    // Generate GEMINI.md context for this change
    let change_dir = project_root.join("agentd/changes").join(change_id);
    crate::context::generate_gemini_context(&change_dir, ContextPhase::Proposal)?;

    println!(
        "{}",
        "🤖 Regenerating proposal with Gemini based on challenge feedback...".cyan()
    );

    let orchestrator = GeminiOrchestrator::new(&config, &project_root);
    let (_output, _usage) = orchestrator.run_reproposal(change_id, complexity).await?;

    println!("\n{}", "✅ Proposal updated!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the updated proposal");
    println!("   2. Re-challenge: agentd challenge {}", change_id);
    println!("   3. Or proceed: agentd implement {}", change_id);

    Ok(())
}
