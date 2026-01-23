use crate::cli::proposal_engine;
use crate::models::AgentdConfig;
use crate::Result;
use colored::Colorize;
use std::env;

pub async fn run(change_id: &str, description: Option<String>, skip_clarify: bool) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;

    println!("{}", "🎯 Agentd Plan Workflow".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    // Determine if this is a new or existing change
    let change_dir = project_root
        .join("agentd/changes")
        .join(change_id);
    let state_path = change_dir.join("STATE.yaml");
    let proposal_path = change_dir.join("proposal.md");

    let is_new_change = !state_path.exists();

    // Check for conflict: existing change directory with proposal.md but no STATE.yaml
    if is_new_change && change_dir.exists() && proposal_path.exists() {
        println!("{}", "⚠️  Conflict detected".yellow().bold());
        println!();
        println!("   Directory '{}' contains proposal.md but no STATE.yaml", change_id);
        println!();
        println!("   This suggests an incomplete or corrupted change. Options:");
        println!("     1. Delete the directory and try again:");
        println!("        rm -rf agentd/changes/{}", change_id);
        println!("        agentd plan-change {} \"<description>\"", change_id);
        println!();
        println!("     2. Or check if this is a valid existing change:");
        println!("        ls agentd/changes/{}", change_id);
        return Ok(());
    }

    // For new changes, require description
    let description = if is_new_change {
        match description {
            Some(desc) => desc,
            None => {
                println!("{}", "❌ Error: Description required for new changes".red().bold());
                println!();
                println!("   Usage:");
                println!("     agentd plan-change {} \"<description>\"", change_id);
                println!();
                println!("   For existing changes, description is optional:");
                println!("     agentd plan-change {}", change_id);
                return Ok(());
            }
        }
    } else {
        description.unwrap_or_default()
    };

    // Check clarifications.md for new changes (unless skip_clarify)
    if is_new_change && !skip_clarify {
        let clarifications_path = project_root
            .join("agentd/changes")
            .join(change_id)
            .join("clarifications.md");

        if !clarifications_path.exists() {
            println!("{}", "❌ Error: clarifications.md not found".red().bold());
            println!();
            println!("   The planning workflow requires clarifications. Run:");
            println!("     /agentd:plan {} \"{}\"", change_id, description);
            println!();
            println!("   Or skip clarifications with:");
            println!("     agentd plan-change {} \"{}\" --skip-clarify", change_id, description);
            return Ok(());
        }
        println!("{}", "✅ clarifications.md found".green());
    } else if skip_clarify {
        println!("{}", "⏭️  Skipping clarification check (--skip-clarify)".yellow());
    } else {
        println!("{}", "ℹ️  Continuing existing change".blue());
    }

    // Delegate to idempotent plan engine
    let engine_config = proposal_engine::ProposalEngineConfig {
        change_id: change_id.to_string(),
        description,
        skip_clarify,
        project_root,
        config,
    };

    proposal_engine::run_plan_change(engine_config).await?;
    Ok(())
}
