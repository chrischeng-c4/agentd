use crate::orchestrator::{ClaudeOrchestrator, UsageMetrics};
use crate::state::StateManager;
use crate::{
    models::{Change, AgentdConfig, Complexity},
    Result,
};
use colored::Colorize;
use std::env;
use std::path::PathBuf;

/// Record LLM usage to StateManager
fn record_usage(
    change_id: &str,
    project_root: &PathBuf,
    step: &str,
    model: &str,
    usage: &UsageMetrics,
    config: &AgentdConfig,
    complexity: Complexity,
) {
    let state_path = project_root
        .join("agentd/changes")
        .join(change_id)
        .join("STATE.yaml");

    if let Ok(mut manager) = StateManager::load(&state_path) {
        let m = config.claude.select_model(complexity);
        manager.record_llm_call(
            step,
            Some(model.to_string()),
            usage.tokens_in,
            usage.tokens_out,
            usage.duration_ms,
            m.cost_per_1m_input,
            m.cost_per_1m_output,
        );
        let _ = manager.save();
    }
}

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
    let (_output, usage) = orchestrator.run_resolve(change_id, complexity).await?;
    let model = config.claude.select_model(complexity).model.clone();
    record_usage(change_id, &project_root, "resolve", &model, &usage, &config, complexity);

    println!("\n{}", "✅ Issues resolved!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the changes made");
    println!("   2. Re-review: agentd review {}", change_id);
    println!("   3. Or test manually and archive if ready");

    Ok(())
}
