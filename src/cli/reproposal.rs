use crate::context::ContextPhase;
use crate::orchestrator::{find_session_index, GeminiOrchestrator, UsageMetrics};
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
        let m = config.gemini.select_model(complexity);
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

    // Load session_id from STATE.yaml for resume-by-index - R2 requires strict enforcement
    let state_path = project_root
        .join("agentd/changes")
        .join(change_id)
        .join("STATE.yaml");

    let session_index = match StateManager::load(&state_path) {
        Ok(manager) => {
            match manager.session_id() {
                Some(sid) => {
                    match find_session_index(sid, &project_root).await {
                        Ok(index) => index,
                        Err(e) => {
                            println!("{}", format!("❌ Session not found, please re-run proposal: {}", e).red().bold());
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    println!("{}", "❌ Failed to capture session ID".red().bold());
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            println!("{}", "❌ Failed to load STATE.yaml".red().bold());
            std::process::exit(1);
        }
    };

    // Use resume-by-index (R2 requires strict enforcement - no fallback to --resume latest)
    let (_output, usage) = orchestrator.run_reproposal_with_session(change_id, session_index, complexity).await?;

    let model = config.gemini.select_model(complexity).model.clone();
    record_usage(change_id, &project_root, "reproposal", &model, &usage, &config, complexity);

    println!("\n{}", "✅ Proposal updated!".green().bold());
    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Review the updated proposal");
    println!("   2. Re-challenge: agentd challenge {}", change_id);
    println!("   3. Or proceed: agentd implement {}", change_id);

    Ok(())
}
