use crate::context::ContextPhase;
use crate::orchestrator::{CodexOrchestrator, UsageMetrics};
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
        let m = config.codex.select_model(complexity);
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

pub struct ChallengeCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;

    // Load config
    let config = AgentdConfig::load(&project_root)?;

    // Check if change exists
    let change_dir = project_root.join("agentd/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Run 'agentd proposal {}' first.",
            change_id,
            change_id
        );
    }

    // Create Change object and validate
    let change = Change::new(change_id, "");
    change.validate_structure(&project_root)?;

    // Assess complexity dynamically based on change structure
    let complexity = change.assess_complexity(&project_root);

    // Generate AGENTS.md context for this change
    crate::context::generate_agents_context(&change_dir, ContextPhase::Challenge)?;

    // No longer need to create CHALLENGE.md skeleton - reviews go in proposal.md

    println!(
        "{}",
        "🔍 Analyzing proposal with Codex...".cyan()
    );

    // Run Codex orchestrator
    let orchestrator = CodexOrchestrator::new(&config, &project_root);
    let (output, usage) = orchestrator.run_challenge(change_id, complexity).await?;
    let model = config.codex.select_model(complexity).model.clone();
    record_usage(change_id, &project_root, "challenge", &model, &usage, &config, complexity);

    println!("\n{}", "📊 Challenge Review Generated".green().bold());

    // Check proposal.md for review block (new format)
    let proposal_path = change_dir.join("proposal.md");
    if !proposal_path.exists() {
        anyhow::bail!("proposal.md not found");
    }

    let proposal_content = std::fs::read_to_string(&proposal_path)?;
    let latest_review = crate::parser::parse_latest_review(&proposal_content)?;

    match latest_review {
        Some(review) => {
            println!("   Location: {} (review block)", proposal_path.display());

            // Display review summary
            display_review_summary(&review.content);

            // Parse verdict from review
            let verdict = match review.status.to_lowercase().as_str() {
                "approved" => crate::models::ChallengeVerdict::Approved,
                "needs_revision" => crate::models::ChallengeVerdict::NeedsRevision,
                "rejected" => crate::models::ChallengeVerdict::Rejected,
                _ => crate::models::ChallengeVerdict::Unknown,
            };

            // Update STATE.yaml
            println!();
            println!("{}", "📝 Updating STATE.yaml...".cyan());
            let state_path = change_dir.join("STATE.yaml");
            let mut state_manager = crate::state::StateManager::load(&state_path)?;
            state_manager.update_phase_from_verdict(&review.status);
            state_manager.update_checksum("proposal.md")?;
            state_manager.save()?;
            println!("   {} Phase updated based on verdict: {:?}", "✓".green(), verdict);

            // Provide next steps based on verdict
            println!("\n{}", "⏭️  Next steps:".yellow());
            match verdict {
                crate::models::ChallengeVerdict::Approved => {
                    println!("   ✅ Proposal approved!");
                    println!("   agentd implement {}", change_id);
                }
                crate::models::ChallengeVerdict::NeedsRevision => {
                    println!("   ⚠️  Needs revision");
                    println!("   agentd reproposal {}", change_id);
                }
                crate::models::ChallengeVerdict::Rejected => {
                    println!("   ❌ Proposal rejected");
                    println!("   Review issues and update manually");
                }
                crate::models::ChallengeVerdict::Unknown => {
                    println!("   ❓ Unknown verdict");
                }
            }
        }
        None => {
            println!("\n{}", "⚠️  Warning: No review found in proposal.md".yellow());
            println!("   Codex may not have appended the review block correctly.");
            println!("\n   Codex output:");
            println!("{}", output);
        }
    }

    Ok(())
}

fn display_review_summary(content: &str) {
    // Parse severity counts from review content
    let high_count = content.matches("**Severity**: High").count();
    let medium_count = content.matches("**Severity**: Medium").count();
    let low_count = content.matches("**Severity**: Low").count();

    if high_count > 0 || medium_count > 0 || low_count > 0 {
        println!(
            "   Issues: {} HIGH, {} MEDIUM, {} LOW",
            high_count, medium_count, low_count
        );
    } else {
        println!("   ✅ No issues found");
    }
}
