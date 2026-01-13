use crate::orchestrator::ScriptRunner;
use crate::{
    models::{Change, SpecterConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub struct VerifyCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = SpecterConfig::load(&project_root)?;

    let change = Change::new(change_id, "");
    change.validate_structure(&project_root)?;

    // Generate AGENTS.md context for this change
    let change_dir = project_root.join("specter/changes").join(change_id);
    crate::context::generate_agents_context(&change_dir)?;

    println!("{}", "🧪 Verifying with Codex...".cyan());

    let script_runner = ScriptRunner::new(config.scripts_dir);
    let _output = script_runner.run_codex_verify(change_id).await?;

    println!("\n{}", "✅ Verification complete!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   specter archive {}", change_id);

    Ok(())
}
