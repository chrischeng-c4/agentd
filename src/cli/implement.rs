use crate::orchestrator::ScriptRunner;
use crate::parser::parse_review_verdict;
use crate::{
    models::{Change, ReviewVerdict, SpecterConfig},
    Result,
};
use colored::Colorize;
use std::env;
use std::path::PathBuf;

pub struct ImplementCommand;

pub async fn run(change_id: &str, tasks: Option<&str>) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = SpecterConfig::load(&project_root)?;

    println!("{}", "🎨 Specter Implementation Workflow".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    // Step 1: Implementation (Claude writes code + tests)
    println!("{}", "🎨 [1/6] Implementing with Claude...".cyan());
    run_implement_step(change_id, tasks, &project_root, &config).await?;

    // Step 2: First review (iteration 0)
    println!();
    println!("{}", "🔍 [2/6] Reviewing with Codex (iteration 0)...".cyan());
    let verdict = run_review_step(change_id, &project_root, &config, 0).await?;

    match verdict {
        ReviewVerdict::Approved => {
            println!();
            println!("{}", "✨ Implementation approved!".green().bold());
            println!("\n{}", "⏭️  Next:".yellow());
            println!("   specter archive {}", change_id);
            return Ok(());
        }
        ReviewVerdict::NeedsChanges => {
            println!();
            println!("{}", "⚠️  NEEDS_CHANGES - Auto-fixing (iteration 1)...".yellow());

            // Step 3: First resolve
            println!();
            println!("{}", "🔧 [3/6] Resolving issues (iteration 1)...".cyan());
            run_resolve_step(change_id, &project_root, &config).await?;

            // Step 4: Second review (iteration 1)
            println!();
            println!("{}", "🔍 [4/6] Re-reviewing (iteration 1)...".cyan());
            let verdict2 = run_review_step(change_id, &project_root, &config, 1).await?;

            match verdict2 {
                ReviewVerdict::Approved => {
                    println!();
                    println!("{}", "✨ Fixed and approved!".green().bold());
                    println!("\n{}", "⏭️  Next:".yellow());
                    println!("   specter archive {}", change_id);
                    return Ok(());
                }
                ReviewVerdict::NeedsChanges => {
                    println!();
                    println!("{}", "⚠️  Still needs changes - Auto-fixing (iteration 2)...".yellow());

                    // Step 5: Second resolve
                    println!();
                    println!("{}", "🔧 [5/6] Resolving issues (iteration 2)...".cyan());
                    run_resolve_step(change_id, &project_root, &config).await?;

                    // Step 6: Final review (iteration 2)
                    println!();
                    println!("{}", "🔍 [6/6] Final review (iteration 2)...".cyan());
                    let verdict3 = run_review_step(change_id, &project_root, &config, 2).await?;

                    match verdict3 {
                        ReviewVerdict::Approved => {
                            println!();
                            println!("{}", "✨ Fixed and approved!".green().bold());
                            println!("\n{}", "⏭️  Next:".yellow());
                            println!("   specter archive {}", change_id);
                            Ok(())
                        }
                        _ => {
                            println!();
                            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                            println!("{}", "⚠️  Automatic refinement limit reached (2 iterations)".yellow().bold());
                            display_remaining_issues(change_id, &project_root)?;
                            Ok(())
                        }
                    }
                }
                ReviewVerdict::MajorIssues => {
                    println!();
                    println!("{}", "❌ Major issues remain after iteration 1".red().bold());
                    display_remaining_issues(change_id, &project_root)?;
                    Ok(())
                }
                ReviewVerdict::Unknown => {
                    println!("{}", "⚠️  Could not parse review verdict".yellow());
                    Ok(())
                }
            }
        }
        ReviewVerdict::MajorIssues => {
            println!();
            println!("{}", "❌ Major issues found".red().bold());
            display_remaining_issues(change_id, &project_root)?;
            Ok(())
        }
        ReviewVerdict::Unknown => {
            println!("{}", "⚠️  Could not parse review verdict".yellow());
            Ok(())
        }
    }
}

/// Run implementation step (Claude writes code + tests)
async fn run_implement_step(
    change_id: &str,
    tasks: Option<&str>,
    project_root: &PathBuf,
    config: &SpecterConfig,
) -> Result<()> {
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    let _output = script_runner
        .run_claude_implement(change_id, tasks)
        .await?;

    println!("{}", "✅ Implementation complete (code + tests written)".green());
    Ok(())
}

/// Run review step with iteration tracking
async fn run_review_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &SpecterConfig,
    iteration: u32,
) -> Result<ReviewVerdict> {
    let change_dir = project_root.join("specter/changes").join(change_id);

    // Regenerate AGENTS.md context
    crate::context::generate_agents_context(&change_dir)?;

    // Create/update REVIEW.md skeleton
    crate::context::create_review_skeleton(&change_dir, change_id, iteration)?;

    // Run Codex review script with iteration number
    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    let _output = script_runner
        .run_codex_review(change_id, iteration)
        .await?;

    // Parse verdict
    let change = Change::new(change_id, "");
    let review_path = change.review_path(project_root);
    let verdict = parse_review_verdict(&review_path)?;

    // Display summary
    display_review_summary(&review_path, &verdict, iteration)?;

    Ok(verdict)
}

/// Run resolve step (Claude fixes issues from review)
async fn run_resolve_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &SpecterConfig,
) -> Result<()> {
    let change = Change::new(change_id, "");
    let review_path = change.review_path(project_root);

    if !review_path.exists() {
        anyhow::bail!("REVIEW.md not found for resolving issues");
    }

    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    let _output = script_runner.run_claude_resolve(change_id).await?;

    println!("{}", "✅ Issues resolved".green());
    Ok(())
}

/// Display review summary after each review
fn display_review_summary(
    review_path: &PathBuf,
    verdict: &ReviewVerdict,
    _iteration: u32,
) -> Result<()> {
    if !review_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(review_path)?;

    // Parse test status
    let test_status = if content.contains("**Overall Status**: ✅ PASS") {
        "✅ PASS"
    } else if content.contains("**Overall Status**: ❌ FAIL") {
        "❌ FAIL"
    } else if content.contains("**Overall Status**: ⚠️ PARTIAL") {
        "⚠️ PARTIAL"
    } else {
        "❓ UNKNOWN"
    };

    // Count issues
    let high_count = content.matches("**Severity**: High").count();
    let medium_count = content.matches("**Severity**: Medium").count();

    println!();
    println!("   Tests: {}", test_status);
    println!("   Issues: {} high, {} medium", high_count, medium_count);
    println!("   Verdict: {}", format_verdict(verdict));

    Ok(())
}

/// Display remaining issues when automatic refinement fails
fn display_remaining_issues(change_id: &str, project_root: &PathBuf) -> Result<()> {
    let change = Change::new(change_id, "");
    let review_path = change.review_path(project_root);

    if !review_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&review_path)?;

    // Count issues
    let high_count = content.matches("**Severity**: High").count();
    let medium_count = content.matches("**Severity**: Medium").count();

    println!();
    println!("{}", "📊 Remaining Issues:".cyan());
    println!("   🔴 High:    {} issues", high_count);
    println!("   🟡 Medium:  {} issues", medium_count);

    println!();
    println!("{}", "⏭️  Next steps:".yellow());
    println!("   1. Review full report:");
    println!("      cat {}", review_path.display());
    println!();
    println!("   2. Fix issues manually and re-review:");
    println!("      specter review {}", change_id);
    println!();
    println!("   3. Or resolve specific issues:");
    println!("      specter resolve-reviews {}", change_id);

    Ok(())
}

fn format_verdict(verdict: &ReviewVerdict) -> colored::ColoredString {
    match verdict {
        ReviewVerdict::Approved => "APPROVED".green().bold(),
        ReviewVerdict::NeedsChanges => "NEEDS_CHANGES".yellow().bold(),
        ReviewVerdict::MajorIssues => "MAJOR_ISSUES".red().bold(),
        ReviewVerdict::Unknown => "UNKNOWN".bright_black(),
    }
}
