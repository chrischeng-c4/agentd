use crate::{Result, models::{Change, SpecterConfig}};
use crate::orchestrator::ScriptRunner;
use colored::Colorize;
use std::env;

pub struct ReproposalCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = SpecterConfig::load(&project_root)?;

    // Check if change and challenge exist
    let change = Change::new(change_id, "");
    let challenge_path = change.challenge_path(&project_root);

    if !challenge_path.exists() {
        anyhow::bail!(
            "No challenge found for '{}'. Run 'specter challenge {}' first.",
            change_id,
            change_id
        );
    }

    println!("{}", "🤖 Regenerating proposal with Gemini based on challenge feedback...".cyan());

    let script_runner = ScriptRunner::new(config.scripts_dir);
    let output = script_runner.run_gemini_reproposal(change_id).await?;

    println!("\n{}", "✅ Proposal updated!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the updated proposal");
    println!("   2. Re-challenge: specter challenge {}", change_id);
    println!("   3. Or proceed: specter implement {}", change_id);

    Ok(())
}
