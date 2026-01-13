use crate::models::{
    decide_merging_strategy, ArchiveReviewVerdict, DeltaMetrics, MergingStrategy, AgentdConfig,
    ValidationRules,
};
use crate::orchestrator::ScriptRunner;
use crate::parser::parse_archive_review_verdict;
use crate::validator::{SemanticValidator, SpecFormatValidator};
use crate::Result;
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ArchiveCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let config = AgentdConfig::load(&project_root)?;

    println!("{}", "📦 Agentd Archive Workflow".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!();

    let agentd_dir = project_root.join("agentd");
    let change_dir = agentd_dir.join("changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", change_id);
    }

    // Step 1: Validate spec files (zero token cost)
    println!("{}", "🔍 [1/7] Validating spec files...".cyan());
    let specs_dir = change_dir.join("specs");
    let validation_result = validate_specs(&specs_dir, &config.validation)?;

    if !validation_result.is_valid() {
        println!();
        println!("{}", "❌ Validation failed:".red().bold());
        for error in validation_result.high_severity_errors() {
            println!("   {}", error.format());
        }
        println!();
        println!("{}", "🛑 Archive blocked. Fix validation errors first.".yellow());
        return Ok(());
    }

    println!("   {} All specs valid", "✅".green());

    // Step 2: Compute metrics and decide strategy (zero token cost)
    println!();
    println!("{}", "📊 [2/7] Analyzing delta metrics...".cyan());

    let spec_files = collect_spec_files(&specs_dir)?;
    if spec_files.is_empty() {
        println!("   ⚠️  No spec files found in specs/ directory");
        println!("   Skipping merge step");
    }

    // Collect merge strategies for all files (for Codex review later)
    let mut merge_strategies = Vec::new();

    for spec_file in &spec_files {
        let relative_path = spec_file.strip_prefix(&specs_dir)?;
        let main_spec_path = agentd_dir.join("specs").join(relative_path);

        let metrics = compute_delta_metrics(&main_spec_path, spec_file)?;
        let decision = decide_merging_strategy(&metrics);

        println!();
        println!("   File: {}", relative_path.display());
        println!(
            "   Strategy: {} {}",
            decision.strategy.emoji(),
            decision.strategy.name()
        );
        println!("   Reason: {}", decision.reason);

        merge_strategies.push((relative_path.to_path_buf(), decision.strategy.clone()));
    }

    // Step 3: Backup original specs (for potential rollback)
    println!();
    println!("{}", "💾 [3/7] Backing up original specs...".cyan());
    backup_original_specs(&project_root)?;
    println!("   {} Backup created", "✅".green());

    // Step 4: Merge specs with Gemini
    println!();
    println!("{}", "🔄 [4/7] Merging spec deltas with Gemini...".cyan());

    for (relative_path, strategy) in &merge_strategies {
        println!();
        println!("   Merging: {}", relative_path.display());

        merge_spec_with_gemini(
            change_id,
            strategy,
            relative_path.to_str().unwrap(),
            &config,
        )
        .await?;

        println!(
            "   {} Merged to agentd/specs/{}",
            "✅".green(),
            relative_path.display()
        );
    }

    // Step 5: Generate CHANGELOG with Gemini
    println!();
    println!("{}", "📝 [5/7] Generating CHANGELOG entry...".cyan());
    generate_changelog_entry(change_id, &project_root, &config).await?;
    println!("   {} CHANGELOG updated", "✅".green());

    // Step 6: Quality review with Codex
    println!();
    println!("{}", "🔍 [6/7] Reviewing with Codex...".cyan());

    // Use first strategy for review script (or "mixed" if multiple)
    let review_strategy = if merge_strategies.len() == 1 {
        merge_strategies[0].1.name()
    } else if merge_strategies.is_empty() {
        "no-merge"
    } else {
        "mixed"
    };

    let script_runner = ScriptRunner::new(config.scripts_dir.clone());
    script_runner
        .run_codex_archive_review(change_id, review_strategy)
        .await?;

    // Parse review verdict
    let review_path = change_dir.join("ARCHIVE_REVIEW.md");
    let verdict = parse_archive_review_verdict(&review_path)?;

    match verdict {
        ArchiveReviewVerdict::Approved => {
            println!("   {} Quality review passed", "✅".green());
        }
        ArchiveReviewVerdict::NeedsFix | ArchiveReviewVerdict::Rejected => {
            println!();
            println!("{}", "❌ Quality review failed".red().bold());
            display_review_issues(&review_path)?;
            println!();
            println!(
                "{}",
                "🛑 Archive blocked. Restoring original specs...".yellow()
            );

            restore_original_specs(&project_root)?;

            println!();
            println!("Fix issues and re-run: agentd archive {}", change_id);
            return Ok(());
        }
        ArchiveReviewVerdict::Unknown => {
            println!("   ⚠️  Could not parse review verdict");
            println!("   Review report: {}", review_path.display());
            println!();
            println!(
                "{}",
                "🛑 Archive blocked due to unknown verdict. Restoring original specs...".yellow()
            );

            restore_original_specs(&project_root)?;

            println!();
            println!("Check review report and re-run: agentd archive {}", change_id);
            return Ok(());
        }
    }

    // Step 7: Move to archive
    println!();
    println!("{}", "📦 [7/7] Moving to archive...".cyan());

    let timestamp = chrono::Local::now().format("%Y%m%d");
    let archive_path = move_to_archive(change_id, &timestamp.to_string(), &project_root)?;

    println!("   {} Archived to: {}", "✅".green(), archive_path.display());

    // Clean up backup after successful archive
    cleanup_backup(&project_root)?;

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!("{}", "✨ Archive complete!".green().bold());
    println!();
    println!("   Main specs: agentd/specs/");
    println!("   Archive: {}", archive_path.display());
    println!("   CHANGELOG: agentd/specs/CHANGELOG.md");

    Ok(())
}

/// Validate all spec files in the directory
fn validate_specs(specs_dir: &Path, rules: &ValidationRules) -> Result<crate::models::ValidationResult> {
    let format_validator = SpecFormatValidator::new(rules.clone());
    let semantic_validator = SemanticValidator::new(rules.clone());

    let mut all_errors = Vec::new();
    let spec_files = collect_spec_files(specs_dir)?;

    // Format validation (per-file)
    for spec_file in &spec_files {
        let result = format_validator.validate(spec_file);
        all_errors.extend(result.errors);
    }

    // Semantic validation (cross-file)
    let semantic_result = semantic_validator.validate_batch(&spec_files);
    all_errors.extend(semantic_result.errors);

    Ok(crate::models::ValidationResult::new(all_errors))
}

/// Collect all .md spec files from directory
fn collect_spec_files(specs_dir: &Path) -> Result<Vec<PathBuf>> {
    if !specs_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(specs_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

/// Compute delta metrics for a spec file
fn compute_delta_metrics(main_spec_path: &Path, delta_spec_path: &Path) -> Result<DeltaMetrics> {
    let mut metrics = DeltaMetrics::new();

    // Get file sizes
    if main_spec_path.exists() {
        let main_content = std::fs::read_to_string(main_spec_path)?;
        metrics.existing_spec_size = main_content.len();
        metrics.existing_req_count = count_requirements(&main_content);
    }

    let delta_content = std::fs::read_to_string(delta_spec_path)?;
    metrics.delta_spec_size = delta_content.len();

    // Analyze deltas (simplified - could be more sophisticated)
    // For now, we'll use simple heuristics
    let delta_req_count = count_requirements(&delta_content);

    if metrics.existing_spec_size == 0 {
        // New file - all additions
        metrics.added_count = delta_req_count;
    } else {
        // Estimate changes based on size difference
        let main_content = std::fs::read_to_string(main_spec_path)?;
        let main_req_count = metrics.existing_req_count;

        if delta_req_count > main_req_count {
            metrics.added_count = delta_req_count - main_req_count;
        }

        if delta_req_count < main_req_count {
            metrics.removed_count = main_req_count - delta_req_count;
        }

        // Detect structural changes
        metrics.has_new_sections = detect_new_sections(&main_content, &delta_content);
        metrics.has_schema_changes = delta_content.contains("## Data Schema")
            || delta_content.contains("## Database Schema");
        metrics.has_api_changes =
            delta_content.contains("## API Endpoints") || delta_content.contains("### API:");

        // Affected requirements (simplified: all non-added requirements)
        metrics.affected_req_count = delta_req_count.saturating_sub(metrics.added_count);
        if metrics.affected_req_count == 0 && metrics.added_count == 0 && delta_req_count > 0 {
            metrics.affected_req_count = delta_req_count; // Modifications
            metrics.modified_count = delta_req_count;
        }
    }

    metrics.calculate_ratios();
    Ok(metrics)
}

/// Count requirements in markdown content
fn count_requirements(content: &str) -> usize {
    content
        .lines()
        .filter(|line| line.starts_with("### R") && line.contains(':'))
        .count()
}

/// Detect if new top-level sections were added
fn detect_new_sections(main_content: &str, delta_content: &str) -> bool {
    let main_sections: Vec<_> = main_content
        .lines()
        .filter(|line| line.starts_with("## "))
        .collect();

    let delta_sections: Vec<_> = delta_content
        .lines()
        .filter(|line| line.starts_with("## "))
        .collect();

    delta_sections.len() > main_sections.len()
}

/// Merge spec delta with Gemini
async fn merge_spec_with_gemini(
    change_id: &str,
    strategy: &MergingStrategy,
    spec_file: &str,
    config: &AgentdConfig,
) -> Result<()> {
    let script_runner = ScriptRunner::new(config.scripts_dir.clone());

    script_runner
        .run_gemini_merge_specs(change_id, strategy.name(), spec_file)
        .await?;

    Ok(())
}

/// Generate CHANGELOG entry with Gemini
async fn generate_changelog_entry(
    change_id: &str,
    project_root: &Path,
    config: &AgentdConfig,
) -> Result<()> {
    let script_runner = ScriptRunner::new(config.scripts_dir.clone());

    script_runner.run_gemini_changelog(change_id).await?;

    // Verify CHANGELOG was updated
    let changelog_path = project_root.join("agentd/specs/CHANGELOG.md");
    if !changelog_path.exists() {
        println!("   ⚠️  CHANGELOG.md not found, creating...");
        std::fs::create_dir_all(changelog_path.parent().unwrap())?;
        std::fs::write(
            &changelog_path,
            format!("# CHANGELOG\n\n## {} ({})\n[Entry generated]\n",
                chrono::Local::now().format("%Y-%m-%d"),
                change_id
            ),
        )?;
    }

    Ok(())
}

/// Move change to archive directory
fn move_to_archive(change_id: &str, timestamp: &str, project_root: &Path) -> Result<PathBuf> {
    let agentd_dir = project_root.join("agentd");
    let change_dir = agentd_dir.join("changes").join(change_id);
    let archive_dir = agentd_dir
        .join("archive")
        .join(format!("{}-{}", timestamp, change_id));

    std::fs::create_dir_all(&archive_dir)?;

    // Clean up dynamically generated context files
    crate::context::cleanup_context_files(&change_dir)?;

    // Move change directory to archive
    std::fs::rename(&change_dir, &archive_dir.join(change_id))?;

    Ok(archive_dir)
}

/// Back up current main specs before merging
fn backup_original_specs(project_root: &Path) -> Result<PathBuf> {
    let specs_dir = project_root.join("agentd/specs");
    let backup_dir = project_root.join(".agentd-backup");

    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)?;
    }

    // Copy entire specs directory
    copy_dir_recursive(&specs_dir, &backup_dir)?;

    Ok(backup_dir)
}

/// Restore specs from backup if review fails
fn restore_original_specs(project_root: &Path) -> Result<()> {
    let specs_dir = project_root.join("agentd/specs");
    let backup_dir = project_root.join(".agentd-backup");

    if !backup_dir.exists() {
        anyhow::bail!("No backup found to restore");
    }

    // Remove corrupted merged specs
    std::fs::remove_dir_all(&specs_dir)?;

    // Restore from backup
    copy_dir_recursive(&backup_dir, &specs_dir)?;

    // Clean up backup
    std::fs::remove_dir_all(&backup_dir)?;

    println!("   {} Original specs restored", "✅".green());
    Ok(())
}

/// Clean up backup after successful archive
fn cleanup_backup(project_root: &Path) -> Result<()> {
    let backup_dir = project_root.join(".agentd-backup");
    if backup_dir.exists() {
        std::fs::remove_dir_all(&backup_dir)?;
    }
    Ok(())
}

/// Recursive directory copy helper
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Display review issues from ARCHIVE_REVIEW.md
fn display_review_issues(review_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(review_path)?;

    // Extract issues section
    if let Some(issues_section) = content.split("## Issues Found").nth(1) {
        if let Some(issues_text) = issues_section.split("##").next() {
            println!();
            println!("{}", "Issues Found:".yellow());
            println!("{}", issues_text.trim());
        }
    }

    println!();
    println!("Full report: {}", review_path.display());
    Ok(())
}
