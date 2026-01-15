use crate::orchestrator::ScriptRunner;
use crate::{
    models::{Change, AgentdConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub struct ResolveReviewsCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;

    // Check if change and review exist
    let change = Change::new(change_id, "");
    let review_path = change.review_path(&project_root);

    if !review_path.exists() {
        anyhow::bail!(
            "No review found for '{}'. Run 'agentd review {}' first.",
            change_id,
            change_id
        );
    }

    println!(
        "{}",
        "🔧 Resolving review issues with Claude...".cyan()
    );

    let script_runner = ScriptRunner::new(config.resolve_scripts_dir(&project_root));
    let _output = script_runner.run_claude_resolve(change_id).await?;

    println!("\n{}", "✅ Issues resolved!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the changes made");
    println!("   2. Re-review: agentd review {}", change_id);
    println!("   3. Or test manually and archive if ready");

    Ok(())
}
