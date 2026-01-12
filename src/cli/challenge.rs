use crate::{Result, models::{Change, ChangePhase, SpecterConfig}};
use crate::orchestrator::ScriptRunner;
use colored::Colorize;
use std::env;

pub struct ChallengeCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;

    // Load config
    let config = SpecterConfig::load(&project_root)?;

    // Check if change exists
    let change_dir = project_root.join("changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found. Run 'specter proposal {}' first.", change_id, change_id);
    }

    // Create Change object and validate
    let mut change = Change::new(change_id, "");
    change.validate_structure(&project_root)?;

    println!("{}", format!("🔍 Analyzing proposal with Codex...").cyan());

    // Run Codex script
    let script_runner = ScriptRunner::new(config.scripts_dir);
    let output = script_runner
        .run_codex_challenge(change_id)
        .await?;

    println!("\n{}", "📊 Challenge Report Generated".green().bold());

    // Check if CHALLENGE.md was created
    let challenge_path = change.challenge_path(&project_root);
    if challenge_path.exists() {
        println!("   Location: {}", challenge_path.display());

        // Try to parse and display summary
        if let Ok(content) = std::fs::read_to_string(&challenge_path) {
            display_challenge_summary(&content);
        }

        println!("\n{}", "⏭️  Next steps:".yellow());
        println!("   1. Review full report:");
        println!("      cat {}", challenge_path.display());
        println!("\n   2. Address issues automatically:");
        println!("      specter reproposal {}", change_id);
        println!("\n   3. Or edit manually and re-challenge:");
        println!("      specter challenge {}", change_id);
    } else {
        println!("\n{}", "⚠️  Warning: CHALLENGE.md not found".yellow());
        println!("   The Codex script may need adjustment.");
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
            let section = &content[issue_start.saturating_sub(200)..issue_start.saturating_add(400).min(content.len())];

            println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
            println!("{}", "🔴 HIGH SEVERITY ISSUE (first)".red().bold());

            // Try to extract title
            if let Some(title_start) = section.rfind("#### Issue") {
                if let Some(title_end) = section[title_start..].find('\n') {
                    let title = &section[title_start..title_start + title_end];
                    println!("{}", title.trim().yellow());
                }
            }

            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
        }
    }
}
