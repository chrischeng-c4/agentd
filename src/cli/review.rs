use crate::context::ContextPhase;
use crate::orchestrator::ScriptRunner;
use crate::parser::parse_review_verdict;
use crate::{
    models::{Change, ReviewVerdict, AgentdConfig},
    Result,
};
use colored::Colorize;
use std::env;

pub struct ReviewCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;

    // Load config
    let config = AgentdConfig::load(&project_root)?;

    // Check if change exists
    let change_dir = project_root.join("agentd/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Run 'agentd implement {}' first.",
            change_id,
            change_id
        );
    }

    // Create Change object and validate
    let change = Change::new(change_id, "");
    change.validate_structure(&project_root)?;

    // Generate AGENTS.md context for this change
    crate::context::generate_agents_context(&change_dir, ContextPhase::Review)?;

    // Create REVIEW.md skeleton for Codex to fill (iteration 0 for standalone review)
    crate::context::create_review_skeleton(&change_dir, change_id, 0)?;

    println!("{}", "🔍 Reviewing implementation with Codex...".cyan());
    println!("   Running tests and security scans...");

    // Run Codex review script
    let script_runner = ScriptRunner::new(config.scripts_dir);
    let output = script_runner.run_codex_review(change_id, 0).await?;

    println!("\n{}", "📊 Code Review Complete".green().bold());

    // Check if REVIEW.md was created
    let review_path = change.review_path(&project_root);
    if review_path.exists() {
        println!("   Location: {}", review_path.display());

        // Parse verdict and display summary
        let verdict = parse_review_verdict(&review_path)?;
        if let Ok(content) = std::fs::read_to_string(&review_path) {
            display_review_summary(&content, &verdict);
        }

        println!("\n{}", "⏭️  Next steps:".yellow());
        match verdict {
            ReviewVerdict::Approved => {
                println!("   ✅ Implementation approved!");
                println!("   Ready to archive:");
                println!("      agentd archive {}", change_id);
            }
            ReviewVerdict::NeedsChanges => {
                println!("   1. Review full report:");
                println!("      cat {}", review_path.display());
                println!("\n   2. Address issues automatically:");
                println!("      agentd resolve-reviews {}", change_id);
                println!("\n   3. Or fix manually and re-review:");
                println!("      agentd review {}", change_id);
            }
            ReviewVerdict::MajorIssues => {
                println!("   ⚠️  Major issues found");
                println!("   1. Review full report:");
                println!("      cat {}", review_path.display());
                println!("\n   2. Fix critical issues manually");
                println!("\n   3. Re-review when fixed:");
                println!("      agentd review {}", change_id);
            }
            ReviewVerdict::Unknown => {
                println!("   ⚠️  Could not determine verdict");
                println!("   Review the report manually:");
                println!("      cat {}", review_path.display());
            }
        }
    } else {
        println!("\n{}", "⚠️  Warning: REVIEW.md not found".yellow());
        println!("   The Codex script may need adjustment.");
        println!("\n   Codex output:");
        println!("{}", output);
    }

    Ok(())
}

fn display_review_summary(content: &str, verdict: &ReviewVerdict) {
    // Parse test results
    let test_status = if content.contains("**Overall Status**: ✅ PASS") {
        "✅ PASS".green()
    } else if content.contains("**Overall Status**: ❌ FAIL") {
        "❌ FAIL".red()
    } else if content.contains("**Overall Status**: ⚠️ PARTIAL") {
        "⚠️ PARTIAL".yellow()
    } else {
        "❓ UNKNOWN".bright_black()
    };

    // Parse security status
    let security_status = if content.contains("**Status**: ✅ CLEAN") {
        "✅ CLEAN".green()
    } else if content.contains("**Status**: ⚠️ WARNINGS") {
        "⚠️ WARNINGS".yellow()
    } else if content.contains("**Status**: ❌ VULNERABILITIES") {
        "❌ VULNERABILITIES".red()
    } else {
        "❓ UNKNOWN".bright_black()
    };

    // Count issues by severity
    let high_count = content.matches("**Severity**: High").count();
    let medium_count = content.matches("**Severity**: Medium").count();
    let low_count = content.matches("**Severity**: Low").count();

    println!("\n{}", "📊 Summary:".cyan());
    println!("   Tests:    {}", test_status);
    println!("   Security: {}", security_status);
    println!("   Verdict:  {}", format_verdict(verdict));

    println!("\n{}", "📝 Issues Found:".cyan());
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
        println!("   ✅ No issues found!");
    }

    // Show first high-severity issue if present
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
            if let Some(title_start) = section.rfind("### Issue") {
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

fn format_verdict(verdict: &ReviewVerdict) -> colored::ColoredString {
    match verdict {
        ReviewVerdict::Approved => "APPROVED".green().bold(),
        ReviewVerdict::NeedsChanges => "NEEDS_CHANGES".yellow().bold(),
        ReviewVerdict::MajorIssues => "MAJOR_ISSUES".red().bold(),
        ReviewVerdict::Unknown => "UNKNOWN".bright_black(),
    }
}
