use crate::orchestrator::ScriptRunner;
use crate::{
    models::{Change, SpecterConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = SpecterConfig::load(&project_root)?;

    let change = Change::new(change_id, "");
    change.validate_structure(&project_root)?;

    // Check that VERIFICATION.md exists
    let verification_path = change.path(&project_root).join("VERIFICATION.md");
    if !verification_path.exists() {
        anyhow::bail!(
            "No VERIFICATION.md found. Run 'specter verify {}' first.",
            change_id
        );
    }

    println!("{}", "🔧 Fixing issues with Claude...".cyan());

    let script_runner = ScriptRunner::new(config.scripts_dir);
    let _output = script_runner.run_claude_fix(change_id).await?;

    println!("\n{}", "✅ Fix complete!".green().bold());
    println!(
        "   Location: {}",
        change.path(&project_root).display().to_string().cyan()
    );

    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the fixes");
    println!("   2. Re-verify: specter verify {}", change_id);

    Ok(())
}
