use crate::context::ContextPhase;
use crate::orchestrator::CodexOrchestrator;
use crate::{
    models::{Change, AgentdConfig, ValidationOptions},
    Result,
};
use colored::Colorize;
use std::env;

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

    // Create CHALLENGE.md skeleton for Codex to fill
    crate::context::create_challenge_skeleton(&change_dir, change_id)?;

    println!(
        "{}",
        "🔍 Analyzing proposal with Codex...".cyan()
    );

    // Run Codex orchestrator
    let orchestrator = CodexOrchestrator::new(&config, &project_root);
    let (output, _usage) = orchestrator.run_challenge(change_id, complexity).await?;

    println!("\n{}", "📊 Challenge Report Generated".green().bold());

    // Check if CHALLENGE.md was created
    let challenge_path = change.challenge_path(&project_root);
    if challenge_path.exists() {
        println!("   Location: {}", challenge_path.display());

        // Try to parse and display summary
        if let Ok(content) = std::fs::read_to_string(&challenge_path) {
            display_challenge_summary(&content);
        }

        // Validate and update phase based on verdict
        println!();
        println!("{}", "📝 Updating STATE.yaml from verdict...".cyan());
        let validation_options = ValidationOptions::new().with_json(true);
        let verdict = match crate::cli::validate_challenge::validate_challenge(
            change_id,
            &project_root,
            &validation_options,
        ) {
            Ok(result) => {
                println!("   {} Phase updated based on verdict: {:?}", "✓".green(), result.verdict);
                result.verdict
            }
            Err(e) => {
                println!("   {} Failed to update phase: {}", "⚠".yellow(), e);
                crate::models::ChallengeVerdict::Unknown
            }
        };

        // Provide next steps based on verdict
        println!("\n{}", "⏭️  Next steps:".yellow());
        println!("   1. Review full report:");
        println!("      cat {}", challenge_path.display());

        match verdict {
            crate::models::ChallengeVerdict::Approved => {
                println!("\n   2. Proceed to implementation:");
                println!("      agentd implement {}", change_id);
            }
            crate::models::ChallengeVerdict::NeedsRevision => {
                println!("\n   2. Address issues automatically:");
                println!("      agentd reproposal {}", change_id);
                println!("\n   3. Or edit manually and re-challenge:");
                println!("      agentd challenge {}", change_id);
            }
            crate::models::ChallengeVerdict::Rejected => {
                println!("\n   2. Review rejection reasons in CHALLENGE.md");
                println!("      This proposal has fundamental issues requiring manual intervention.");
                println!("\n   3. Consider creating a new proposal with a different approach.");
            }
            crate::models::ChallengeVerdict::Unknown => {
                println!("\n   2. Address issues automatically:");
                println!("      agentd reproposal {}", change_id);
                println!("\n   3. Or edit manually and re-challenge:");
                println!("      agentd challenge {}", change_id);
            }
        }
    } else {
        println!("\n{}", "⚠️  Warning: CHALLENGE.md not found".yellow());
        println!("   The Codex orchestrator may need adjustment.");
        println!("\n   Codex output:");
        println!("{}", output);
    }

    Ok(())
}

fn display_challenge_summary(content: &str) {
    // Parse basic statistics from CHALLENGE.md
    let high_count = content.matches("**Severity**: High").count();
    let medium_count = content.matches("**Severity**: Medium").count();
    let low_count = content.matches("**Severity**: Low").count();

    println!("\n{}", "📊 Summary:".cyan());

    if high_count > 0 {
        println!("   🔴 High:    {} issues", high_count);
    }
    if medium_count > 0 {
        println!("   🟡 Medium:  {} issues", medium_count);
    }
    if low_count > 0 {
        println!("   🟢 Low:     {} issues", low_count);
    }

    if high_count == 0 && medium_count == 0 && low_count == 0 {
        println!("   ✅ No critical issues found!");
    }

    // Try to extract first high-severity issue
    if high_count > 0 {
        if let Some(issue_start) = content.find("**Severity**: High") {
            let section = &content[issue_start.saturating_sub(200)
                ..issue_start.saturating_add(400).min(content.len())];

            println!(
                "\n{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black()
            );
            println!("{}", "🔴 HIGH SEVERITY ISSUE (first)".red().bold());

            // Try to extract title
            if let Some(title_start) = section.rfind("#### Issue") {
                if let Some(title_end) = section[title_start..].find('\n') {
                    let title = &section[title_start..title_start + title_end];
                    println!("{}", title.trim().yellow());
                }
            }

            println!(
                "{}",
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black()
            );
        }
    }
}
