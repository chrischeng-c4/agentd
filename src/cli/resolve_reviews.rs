use crate::orchestrator::ScriptRunner;
use crate::{
    models::{Change, SpecterConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub struct ResolveReviewsCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = SpecterConfig::load(&project_root)?;

    // Check if change and review exist
    let change = Change::new(change_id, "");
    let review_path = change.review_path(&project_root);

    if !review_path.exists() {
        anyhow::bail!(
            "No review found for '{}'. Run 'specter review {}' first.",
            change_id,
            change_id
        );
    }

    println!(
        "{}",
        "🔧 Resolving review issues with Claude...".cyan()
    );

    let script_runner = ScriptRunner::new(config.scripts_dir);
    let _output = script_runner.run_claude_resolve(change_id).await?;

    println!("\n{}", "✅ Issues resolved!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the changes made");
    println!("   2. Re-review: specter review {}", change_id);
    println!("   3. Or test manually and archive if ready");

    Ok(())
}
