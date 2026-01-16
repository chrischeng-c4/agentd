use crate::orchestrator::ClaudeOrchestrator;
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

    // Assess complexity dynamically based on change structure
    let complexity = change.assess_complexity(&project_root);

    println!(
        "{}",
        "🔧 Resolving review issues with Claude...".cyan()
    );

    let orchestrator = ClaudeOrchestrator::new(&config, &project_root);
    let (_output, _usage) = orchestrator.run_resolve(change_id, complexity).await?;

    println!("\n{}", "✅ Issues resolved!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the changes made");
    println!("   2. Re-review: agentd review {}", change_id);
    println!("   3. Or test manually and archive if ready");

    Ok(())
}
