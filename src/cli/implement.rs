use crate::context::ContextPhase;
use crate::models::frontmatter::StatePhase;
use crate::models::{SpecGroup, TaskGraph};
use crate::orchestrator::{ClaudeOrchestrator, CodexOrchestrator, UsageMetrics};
use crate::parser::parse_review_verdict;
use crate::state::StateManager;
use crate::{
    models::{Change, Complexity, ReviewVerdict, AgentdConfig},
    Result,
};
use colored::Colorize;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

/// Result from running the implementation workflow
pub struct ImplementEngineResult {
    pub change_id: String,
    pub final_verdict: ReviewVerdict,
    pub iteration_count: u32,
    pub phase: StatePhase,
}

/// Run implementation for a single spec
async fn run_spec_implementation(
    change_id: &str,
    spec_group: &SpecGroup,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<()> {
    let change = Change::new(change_id, "");
    let complexity = change.assess_complexity(project_root);

    let orchestrator = ClaudeOrchestrator::new(config, project_root);
    let (_output, usage) = orchestrator
        .run_implement_spec(change_id, &spec_group.spec_id, complexity)
        .await?;

    let model = config.claude.select_model(complexity).model.clone();
    record_usage(
        change_id,
        project_root,
        &format!("implement_spec_{}", spec_group.spec_id),
        &model,
        &usage,
        config,
        complexity,
        "claude",
    );

    println!("   ✅ Spec {} implemented", spec_group.spec_id);
    Ok(())
}

/// Run self-review for a single spec
async fn run_spec_self_review(
    change_id: &str,
    spec_group: &SpecGroup,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<bool> {
    let change = Change::new(change_id, "");
    let complexity = change.assess_complexity(project_root);

    let orchestrator = ClaudeOrchestrator::new(config, project_root);
    let (output, usage) = orchestrator
        .run_self_review_spec(change_id, &spec_group.spec_id, complexity)
        .await?;

    let model = config.claude.select_model(complexity).model.clone();
    record_usage(
        change_id,
        project_root,
        &format!("self_review_spec_{}", spec_group.spec_id),
        &model,
        &usage,
        config,
        complexity,
        "claude",
    );

    // Parse self-review verdict - look for markers
    let is_ok = output.contains("✅") || output.to_lowercase().contains("pass");

    if is_ok {
        println!("   ✅ Self-review passed");
    } else {
        println!("   ⚠️  Self-review found issues");
    }

    Ok(is_ok)
}

/// Fix issues found in spec self-review
async fn run_spec_fix(
    change_id: &str,
    spec_group: &SpecGroup,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<()> {
    let change = Change::new(change_id, "");
    let complexity = change.assess_complexity(project_root);

    let orchestrator = ClaudeOrchestrator::new(config, project_root);
    let (_output, usage) = orchestrator
        .run_resolve_spec(change_id, &spec_group.spec_id, complexity)
        .await?;

    let model = config.claude.select_model(complexity).model.clone();
    record_usage(
        change_id,
        project_root,
        &format!("fix_spec_{}", spec_group.spec_id),
        &model,
        &usage,
        config,
        complexity,
        "claude",
    );

    println!("   ✅ Issues fixed");
    Ok(())
}

/// Record LLM usage to StateManager
fn record_usage(
    change_id: &str,
    project_root: &PathBuf,
    step: &str,
    model: &str,
    usage: &UsageMetrics,
    config: &AgentdConfig,
    complexity: Complexity,
    provider: &str,
) {
    let state_path = project_root
        .join("agentd/changes")
        .join(change_id)
        .join("STATE.yaml");

    if let Ok(mut manager) = StateManager::load(&state_path) {
        let (cost_in, cost_out) = match provider {
            "claude" => {
                let m = config.claude.select_model(complexity);
                (m.cost_per_1m_input, m.cost_per_1m_output)
            }
            "codex" => {
                let m = config.codex.select_model(complexity);
                (m.cost_per_1m_input, m.cost_per_1m_output)
            }
            _ => (None, None),
        };
        manager.record_llm_call(
            step,
            Some(model.to_string()),
            usage.tokens_in,
            usage.tokens_out,
            usage.duration_ms,
            cost_in,
            cost_out,
        );
        let _ = manager.save();
    }
}

pub struct ImplementCommand;

/// Run spec-by-spec sequential implementation
pub async fn run_sequential(change_id: &str) -> Result<ImplementEngineResult> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;
    let change_dir = project_root.join("agentd/changes").join(change_id);
    let tasks_path = change_dir.join("tasks.md");

    // Update STATE to Implementing phase
    let mut state_manager = StateManager::load(&change_dir)?;
    state_manager.set_phase(StatePhase::Implementing);
    state_manager.save()?;

    println!("{}", "🎨 Agentd Spec-by-Spec Implementation".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    // 1. Parse task graph
    println!("{}", "📋 Parsing tasks.md...".cyan());
    let task_graph = TaskGraph::from_tasks_file(&tasks_path)?;
    let execution_order = task_graph.get_execution_order();

    let total_specs: usize = task_graph.layers.iter().map(|l| l.specs.len()).sum();
    println!("   Found {} layers, {} specs", task_graph.layers.len(), total_specs);
    println!();

    // 2. Track completed specs
    let mut completed = HashSet::new();

    // 3. Execute spec by spec
    for (idx, spec_group) in execution_order.iter().enumerate() {
        println!(
            "{}",
            format!("⚡ [{}/{}] Implementing spec: {}", idx + 1, total_specs, spec_group.spec_id)
                .cyan()
                .bold()
        );

        // Check prerequisites
        if !task_graph.can_execute_spec(&spec_group.spec_id, &completed) {
            anyhow::bail!("Prerequisites not met for {}", spec_group.spec_id);
        }

        // Implement this spec's tasks
        run_spec_implementation(change_id, spec_group, &project_root, &config).await?;

        // Self-review
        let review_ok = run_spec_self_review(change_id, spec_group, &project_root, &config).await?;

        if !review_ok {
            // Auto-fix issues
            println!("   🔧 Auto-fixing issues...");
            run_spec_fix(change_id, spec_group, &project_root, &config).await?;
        }

        // Mark complete
        completed.insert(spec_group.spec_id.clone());
        println!();
    }

    // 4. Final review by Codex (all specs)
    println!("{}", "🔍 Final review by Codex...".cyan().bold());
    let verdict = run_review_step(change_id, &project_root, &config, 0).await?;

    // Implementation iteration loop (same as current implementation)
    let max_iterations = config.workflow.implementation_iterations;
    let mut current_verdict = verdict;
    let mut iteration = 0;

    loop {
        match current_verdict {
            ReviewVerdict::Approved => {
                // Update STATE to Complete phase
                let mut state_manager = StateManager::load(&change_dir)?;
                state_manager.set_phase(StatePhase::Complete);
                state_manager.save()?;

                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                if iteration == 0 {
                    println!("{}", "✨ Implementation approved!".green().bold());
                } else {
                    println!(
                        "{}",
                        format!("✨ Fixed and approved (iteration {})!", iteration)
                            .green()
                            .bold()
                    );
                }

                // Return result - Skill will use AskUserQuestion for next action
                return Ok(ImplementEngineResult {
                    change_id: change_id.to_string(),
                    final_verdict: ReviewVerdict::Approved,
                    iteration_count: iteration,
                    phase: StatePhase::Complete,
                });
            }
            ReviewVerdict::NeedsChanges => {
                iteration += 1;
                if iteration > max_iterations {
                    println!();
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                    println!(
                        "{}",
                        format!(
                            "⚠️  Automatic refinement limit reached ({} iterations)",
                            max_iterations
                        )
                        .yellow()
                        .bold()
                    );
                    display_remaining_issues(change_id, &project_root)?;

                    // Return result - Skill will use AskUserQuestion for next action
                    return Ok(ImplementEngineResult {
                        change_id: change_id.to_string(),
                        final_verdict: ReviewVerdict::NeedsChanges,
                        iteration_count: iteration - 1,
                        phase: StatePhase::Implementing,
                    });
                }

                println!();
                println!(
                    "{}",
                    format!("⚠️  NEEDS_CHANGES - Auto-fixing (iteration {})...", iteration).yellow()
                );

                // Resolve with Claude
                println!();
                println!("{}", format!("🔧 Resolving issues (iteration {})...", iteration).cyan());
                run_resolve_step(change_id, &project_root, &config).await?;

                // Re-review with Codex
                println!();
                println!("{}", format!("🔍 Re-reviewing (iteration {})...", iteration).cyan());
                current_verdict = run_review_step(change_id, &project_root, &config, iteration).await?;
            }
            ReviewVerdict::MajorIssues => {
                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                println!("{}", "❌ Major issues found".red().bold());
                display_remaining_issues(change_id, &project_root)?;

                // Return result - Skill will use AskUserQuestion for next action
                return Ok(ImplementEngineResult {
                    change_id: change_id.to_string(),
                    final_verdict: ReviewVerdict::MajorIssues,
                    iteration_count: iteration,
                    phase: StatePhase::Implementing,
                });
            }
            ReviewVerdict::Unknown => {
                println!("{}", "⚠️  Could not parse review verdict".yellow());

                // Return result - Skill will use AskUserQuestion for next action
                return Ok(ImplementEngineResult {
                    change_id: change_id.to_string(),
                    final_verdict: ReviewVerdict::Unknown,
                    iteration_count: iteration,
                    phase: StatePhase::Implementing,
                });
            }
        }
    }
}

pub async fn run(change_id: &str, tasks: Option<&str>) -> Result<ImplementEngineResult> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;

    // Check if sequential mode enabled
    let sequential_mode = config.workflow.sequential_implementation;

    if sequential_mode && tasks.is_none() {
        // Use sequential spec-by-spec implementation
        return run_sequential(change_id).await;
    }

    // Legacy: all-at-once implementation
    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Update STATE to Implementing phase
    let mut state_manager = StateManager::load(&change_dir)?;
    state_manager.set_phase(StatePhase::Implementing);
    state_manager.save()?;

    println!("{}", "🎨 Agentd Implementation Workflow".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    // Step 1: Implementation (Claude writes code + tests)
    println!("{}", "🎨 [1/N] Implementing with Claude...".cyan());
    run_implement_step(change_id, tasks, &project_root, &config).await?;

    // Step 2: First review (iteration 0)
    println!();
    println!("{}", "🔍 [2/N] Reviewing with Codex (iteration 0)...".cyan());
    let verdict = run_review_step(change_id, &project_root, &config, 0).await?;

    // Implementation iteration loop
    let max_iterations = config.workflow.implementation_iterations;
    let mut current_verdict = verdict;
    let mut iteration = 0;

    loop {
        match current_verdict {
            ReviewVerdict::Approved => {
                // Update STATE to Complete phase
                let mut state_manager = StateManager::load(&change_dir)?;
                state_manager.set_phase(StatePhase::Complete);
                state_manager.save()?;

                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                if iteration == 0 {
                    println!("{}", "✨ Implementation approved!".green().bold());
                } else {
                    println!("{}", format!("✨ Fixed and approved (iteration {})!", iteration).green().bold());
                }

                // Return result - Skill will use AskUserQuestion for next action
                return Ok(ImplementEngineResult {
                    change_id: change_id.to_string(),
                    final_verdict: ReviewVerdict::Approved,
                    iteration_count: iteration,
                    phase: StatePhase::Complete,
                });
            }
            ReviewVerdict::NeedsChanges => {
                iteration += 1;
                if iteration > max_iterations {
                    println!();
                    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                    println!(
                        "{}",
                        format!("⚠️  Automatic refinement limit reached ({} iterations)", max_iterations).yellow().bold()
                    );
                    display_remaining_issues(change_id, &project_root)?;

                    // Return result - Skill will use AskUserQuestion for next action
                    return Ok(ImplementEngineResult {
                        change_id: change_id.to_string(),
                        final_verdict: ReviewVerdict::NeedsChanges,
                        iteration_count: iteration - 1,
                        phase: StatePhase::Implementing,
                    });
                }

                println!();
                println!(
                    "{}",
                    format!("⚠️  NEEDS_CHANGES - Auto-fixing (iteration {})...", iteration).yellow()
                );

                // Resolve with Claude
                println!();
                println!("{}", format!("🔧 Resolving issues (iteration {})...", iteration).cyan());
                run_resolve_step(change_id, &project_root, &config).await?;

                // Re-review with Codex
                println!();
                println!("{}", format!("🔍 Re-reviewing (iteration {})...", iteration).cyan());
                current_verdict = run_review_step(change_id, &project_root, &config, iteration).await?;
            }
            ReviewVerdict::MajorIssues => {
                println!();
                println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
                println!("{}", "❌ Major issues found".red().bold());
                display_remaining_issues(change_id, &project_root)?;

                // Return result - Skill will use AskUserQuestion for next action
                return Ok(ImplementEngineResult {
                    change_id: change_id.to_string(),
                    final_verdict: ReviewVerdict::MajorIssues,
                    iteration_count: iteration,
                    phase: StatePhase::Implementing,
                });
            }
            ReviewVerdict::Unknown => {
                println!("{}", "⚠️  Could not parse review verdict".yellow());

                // Return result - Skill will use AskUserQuestion for next action
                return Ok(ImplementEngineResult {
                    change_id: change_id.to_string(),
                    final_verdict: ReviewVerdict::Unknown,
                    iteration_count: iteration,
                    phase: StatePhase::Implementing,
                });
            }
        }
    }
}

/// Run implementation step (Claude writes code + tests)
async fn run_implement_step(
    change_id: &str,
    tasks: Option<&str>,
    project_root: &PathBuf,
    config: &AgentdConfig,
) -> Result<()> {
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Assess complexity dynamically based on change structure
    let complexity = change.assess_complexity(project_root);

    let orchestrator = ClaudeOrchestrator::new(config, project_root);
    let (_output, usage) = orchestrator
        .run_implement(change_id, tasks, complexity)
        .await?;
    let model = config.claude.select_model(complexity).model.clone();
    record_usage(change_id, project_root, "implement", &model, &usage, config, complexity, "claude");

    println!("{}", "✅ Implementation complete (code + tests written)".green());
    Ok(())
}

/// Run review step with iteration tracking
async fn run_review_step(
    change_id: &str,
    project_root: &PathBuf,
    config: &AgentdConfig,
    iteration: u32,
) -> Result<ReviewVerdict> {
    let change_dir = project_root.join("agentd/changes").join(change_id);

    // Assess complexity dynamically based on change structure
    let change = Change::new(change_id, "");
    let complexity = change.assess_complexity(project_root);

    // Regenerate AGENTS.md context
    crate::context::generate_agents_context(&change_dir, ContextPhase::Review)?;

    // Create/update REVIEW.md skeleton
    crate::context::create_review_skeleton(&change_dir, change_id, iteration)?;

    // Run Codex review orchestrator with iteration number
    let orchestrator = CodexOrchestrator::new(config, project_root);
    let (_output, usage) = orchestrator
        .run_review(change_id, iteration, complexity)
        .await?;
    let model = config.codex.select_model(complexity).model.clone();
    record_usage(change_id, project_root, "review", &model, &usage, config, complexity, "codex");

    // Parse verdict
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
    config: &AgentdConfig,
) -> Result<()> {
    let change = Change::new(change_id, "");
    let review_path = change.review_path(project_root);

    if !review_path.exists() {
        anyhow::bail!("REVIEW.md not found for resolving issues");
    }

    // Assess complexity dynamically based on change structure
    let complexity = change.assess_complexity(project_root);

    let orchestrator = ClaudeOrchestrator::new(config, project_root);
    let (_output, usage) = orchestrator.run_resolve(change_id, complexity).await?;
    let model = config.claude.select_model(complexity).model.clone();
    record_usage(change_id, project_root, "resolve", &model, &usage, config, complexity, "claude");

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
    println!("      agentd review {}", change_id);
    println!();
    println!("   3. Or resolve specific issues:");
    println!("      agentd resolve-reviews {}", change_id);

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
