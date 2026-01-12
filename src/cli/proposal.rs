use crate::orchestrator::ScriptRunner;
use crate::{
    models::{Change, ChangePhase, SpecterConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub struct ProposalCommand;

pub async fn run(change_id: &str, description: &str) -> Result<()> {
    let project_root = env::current_dir()?;

    // Load config
    let config = SpecterConfig::load(&project_root)?;

    // Create change directory
    let changes_dir = project_root.join("specter/changes");
    std::fs::create_dir_all(&changes_dir)?;

    let change_dir = changes_dir.join(change_id);
    if change_dir.exists() {
        anyhow::bail!("Change '{}' already exists at {:?}", change_id, change_dir);
    }

    std::fs::create_dir_all(&change_dir)?;

    println!("{}", "🤖 Calling Gemini to generate proposal...".cyan());

    // Run Gemini script
    let script_runner = ScriptRunner::new(config.scripts_dir);
    let output = script_runner
        .run_gemini_proposal(change_id, description)
        .await?;

    println!("\n{}", "✨ Gemini output:".green());
    println!("{}", output);

    // Create Change object
    let mut change = Change::new(change_id, description);
    change.update_phase(ChangePhase::Proposed);

    // Validate structure
    match change.validate_structure(&project_root) {
        Ok(_) => {
            println!("\n{}", "✅ Proposal created successfully!".green().bold());
            println!("   Location: {}", change_dir.display());
            println!("\n{}", "📄 Files generated:".cyan());
            println!("   • proposal.md");
            println!("   • tasks.md");
            println!("   • diagrams.md");
            println!("   • specs/<capability>/spec.md");

            println!("\n{}", "⏭️  Next steps:".yellow());
            println!("   1. Review the proposal");
            println!("   2. Run: specter challenge {}", change_id);
        }
        Err(e) => {
            println!(
                "\n{}",
                "⚠️  Warning: Proposal structure incomplete".yellow()
            );
            println!("   {}", e);
            println!("\n   The Gemini script may need adjustment.");
        }
    }

    Ok(())
}
