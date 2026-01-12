use crate::orchestrator::ScriptRunner;
use crate::{
    models::{Change, SpecterConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub struct ImplementCommand;

pub async fn run(change_id: &str, tasks: Option<&str>) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = SpecterConfig::load(&project_root)?;

    let change = Change::new(change_id, "");
    change.validate_structure(&project_root)?;

    println!("{}", "🎨 Implementing with Claude...".cyan());

    let script_runner = ScriptRunner::new(config.scripts_dir);
    let _output = script_runner.run_claude_implement(change_id, tasks).await?;

    println!("\n{}", "✅ Implementation complete!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   specter verify {}", change_id);

    Ok(())
}
