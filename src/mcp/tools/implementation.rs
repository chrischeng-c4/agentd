//! Implementation Support MCP Tools
//!
//! Tools for reading requirements and implementation summaries to support
//! the implementation and review workflow stages.

use super::{get_optional_string, get_required_array, get_required_string, ToolDefinition};
use crate::services::implementation_service::{
    CreateReviewInput, ReviewIssue, ReviewVerdict, Severity, TestResults,
};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

/// Validate change_id to prevent directory traversal attacks
fn validate_change_id(change_id: &str) -> Result<()> {
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!(
            "Invalid change_id '{}': must be lowercase alphanumeric with hyphens only",
            change_id
        );
    }
    if change_id.contains("..") || change_id.starts_with('/') || change_id.starts_with('\\') {
        anyhow::bail!(
            "Invalid change_id '{}': directory traversal not allowed",
            change_id
        );
    }
    Ok(())
}

/// Check if current directory is a git repository
fn is_git_repo() -> bool {
    Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get current git branch name
fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to get current branch: {}", stderr);
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Run a git command and return output
fn run_git_command(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(format!("⚠️ Git command failed: {}", stderr.trim()));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Validate that current branch matches expected pattern and return (branch, is_match)
fn validate_branch(change_id: &str) -> Result<(String, bool)> {
    let current_branch = get_current_branch()?;
    let expected_branch = format!("agentd/{}", change_id);
    let is_match = current_branch == expected_branch;
    Ok((current_branch, is_match))
}

// ============================================================================
// Tool 1: read_all_requirements
// ============================================================================

/// Get the tool definition for read_all_requirements
pub fn read_all_requirements_definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_all_requirements".to_string(),
        description: "Read all requirement files (proposal, tasks, specs) for a change in one call"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID to read requirements for"
                }
            }
        }),
    }
}

/// Execute the read_all_requirements tool
pub fn execute_read_all_requirements(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let change_dir = project_root.join("agentd/changes").join(&change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found at {}", change_id, change_dir.display());
    }

    let mut output = String::new();
    output.push_str(&format!("# Requirements for Change: {}\n\n", change_id));

    // Read proposal.md (required)
    let proposal_path = change_dir.join("proposal.md");
    if !proposal_path.exists() {
        anyhow::bail!("proposal.md not found for change '{}'", change_id);
    }
    let proposal_content = std::fs::read_to_string(&proposal_path)?;
    output.push_str("## Proposal\n\n");
    output.push_str(&proposal_content);
    output.push_str("\n\n---\n\n");

    // Read tasks.md (required)
    let tasks_path = change_dir.join("tasks.md");
    if !tasks_path.exists() {
        anyhow::bail!("tasks.md not found for change '{}'", change_id);
    }
    let tasks_content = std::fs::read_to_string(&tasks_path)?;
    output.push_str("## Tasks\n\n");
    output.push_str(&tasks_content);
    output.push_str("\n\n---\n\n");

    // Read all specs (optional)
    let specs_dir = change_dir.join("specs");
    let mut spec_count = 0;
    if specs_dir.exists() {
        let mut spec_files = Vec::new();
        for entry in std::fs::read_dir(&specs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_stem() {
                    let name_str = name.to_string_lossy();
                    // Skip skeleton files
                    if !name_str.starts_with('_') {
                        spec_files.push((name_str.to_string(), path));
                    }
                }
            }
        }

        spec_files.sort_by(|a, b| a.0.cmp(&b.0));

        if !spec_files.is_empty() {
            output.push_str("## Specifications\n\n");
            for (name, path) in spec_files {
                let spec_content = std::fs::read_to_string(&path)?;
                output.push_str(&format!("### Spec: {}\n\n", name));
                output.push_str(&spec_content);
                output.push_str("\n\n");
                spec_count += 1;
            }
            output.push_str("---\n\n");
        }
    }

    // Summary
    output.push_str(&format!(
        "**Total**: 1 proposal, 1 tasks file, {} specification(s)\n",
        spec_count
    ));

    Ok(output)
}

// ============================================================================
// Tool 2: read_implementation_summary
// ============================================================================

/// Get the tool definition for read_implementation_summary
pub fn read_implementation_summary_definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_implementation_summary".to_string(),
        description: "Get git diff summary and commit log for implementation review".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID to get implementation summary for"
                },
                "base_branch": {
                    "type": "string",
                    "description": "Base branch to compare against (default: 'main')",
                    "default": "main"
                }
            }
        }),
    }
}

/// Execute the read_implementation_summary tool
pub fn execute_read_implementation_summary(args: &Value, _project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let base_branch = get_optional_string(args, "base_branch").unwrap_or_else(|| "main".to_string());

    if !is_git_repo() {
        anyhow::bail!("Not in a git repository");
    }

    let mut output = String::new();
    output.push_str(&format!("# Implementation Summary for: {}\n\n", change_id));

    // Branch validation
    match validate_branch(&change_id) {
        Ok((current_branch, is_match)) => {
            output.push_str(&format!("**Current Branch**: `{}`\n", current_branch));
            if !is_match {
                output.push_str(&format!(
                    "⚠️ **Warning**: Expected branch `agentd/{}` but on `{}`\n",
                    change_id, current_branch
                ));
            }
            output.push_str("\n");
        }
        Err(e) => {
            output.push_str(&format!("⚠️ **Warning**: Could not verify branch: {}\n\n", e));
        }
    }

    // Commits ahead of base
    let commits_ahead = run_git_command(&[
        "rev-list",
        "--count",
        &format!("{}..HEAD", base_branch),
    ])?;
    output.push_str(&format!("**Commits ahead of {}**: {}\n\n", base_branch, commits_ahead));

    // Changed files (name-status)
    output.push_str("## Changed Files\n\n");
    let name_status = run_git_command(&["diff", "--name-status", &base_branch])?;
    if name_status.is_empty() {
        output.push_str("*No changes detected*\n\n");
    } else {
        output.push_str("```\n");
        output.push_str(&name_status);
        output.push_str("\n```\n\n");
    }

    // Diff statistics
    output.push_str("## Diff Statistics\n\n");
    let diff_stat = run_git_command(&["diff", "--stat", &base_branch])?;
    if diff_stat.is_empty() {
        output.push_str("*No changes*\n\n");
    } else {
        output.push_str("```\n");
        output.push_str(&diff_stat);
        output.push_str("\n```\n\n");
    }

    // Commit log
    output.push_str("## Commit Log\n\n");
    let commit_log = run_git_command(&[
        "log",
        "--oneline",
        &format!("{}..HEAD", base_branch),
    ])?;
    if commit_log.is_empty() {
        output.push_str("*No commits*\n\n");
    } else {
        output.push_str("```\n");
        output.push_str(&commit_log);
        output.push_str("\n```\n\n");
    }

    output.push_str("---\n\n");
    output.push_str("💡 **Note**: For detailed code review, use the `Read` tool to examine specific files.\n");

    Ok(output)
}

// ============================================================================
// Tool 3: list_changed_files
// ============================================================================

/// Get the tool definition for list_changed_files
pub fn list_changed_files_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_changed_files".to_string(),
        description: "List changed files with detailed statistics (additions/deletions)".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID to list files for"
                },
                "base_branch": {
                    "type": "string",
                    "description": "Base branch to compare against (default: 'main')",
                    "default": "main"
                },
                "filter": {
                    "type": "string",
                    "description": "Optional filter pattern (simple string match on file path)"
                }
            }
        }),
    }
}

/// Execute the list_changed_files tool
pub fn execute_list_changed_files(args: &Value, _project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let base_branch = get_optional_string(args, "base_branch").unwrap_or_else(|| "main".to_string());
    let filter = get_optional_string(args, "filter");

    if !is_git_repo() {
        anyhow::bail!("Not in a git repository");
    }

    let mut output = String::new();
    output.push_str(&format!("# Changed Files for: {}\n\n", change_id));

    if let Some(ref f) = filter {
        output.push_str(&format!("**Filter**: `{}`\n\n", f));
    }

    // Get numstat output
    let numstat = run_git_command(&["diff", "--numstat", &base_branch])?;

    if numstat.is_empty() || numstat.starts_with("⚠️") {
        output.push_str("*No changes detected*\n");
        return Ok(output);
    }

    // Parse numstat output
    #[derive(Debug)]
    struct FileStat {
        added: String,
        removed: String,
        path: String,
    }

    let mut files: Vec<FileStat> = Vec::new();
    let mut total_added = 0;
    let mut total_removed = 0;

    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 3 {
            continue;
        }

        let path = parts[2].to_string();

        // Apply filter if specified
        if let Some(ref f) = filter {
            if !path.contains(f) {
                continue;
            }
        }

        let added_str = parts[0].to_string();
        let removed_str = parts[1].to_string();

        // Parse numbers (handle binary files marked with '-')
        if added_str != "-" {
            if let Ok(n) = added_str.parse::<usize>() {
                total_added += n;
            }
        }
        if removed_str != "-" {
            if let Ok(n) = removed_str.parse::<usize>() {
                total_removed += n;
            }
        }

        files.push(FileStat {
            added: added_str,
            removed: removed_str,
            path,
        });
    }

    if files.is_empty() {
        output.push_str("*No matching files found*\n");
        return Ok(output);
    }

    // Format as markdown table
    output.push_str("| File | Status | +Lines | -Lines |\n");
    output.push_str("|------|--------|--------|--------|\n");

    for file in &files {
        let status = if file.added == "-" && file.removed == "-" {
            "Binary"
        } else if file.removed == "0" {
            "Added"
        } else if file.added == "0" {
            "Deleted"
        } else {
            "Modified"
        };

        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            file.path, status, file.added, file.removed
        ));
    }

    output.push_str("\n");
    output.push_str(&format!(
        "**Totals**: {} files, +{} lines, -{} lines\n",
        files.len(),
        total_added,
        total_removed
    ));

    Ok(output)
}

// ============================================================================
// Tool 4: create_review
// ============================================================================

/// Get the tool definition for create_review
pub fn create_review_definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_review".to_string(),
        description: "Create structured REVIEW.md file with test results, issues, and verdict"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id", "iteration", "verdict", "issues"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID being reviewed"
                },
                "iteration": {
                    "type": "integer",
                    "description": "Review iteration number (starts at 0)"
                },
                "test_results": {
                    "type": "object",
                    "description": "Test execution results",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["PASS", "FAIL", "PARTIAL", "UNKNOWN"],
                            "description": "Overall test status"
                        },
                        "total": { "type": "integer" },
                        "passed": { "type": "integer" },
                        "failed": { "type": "integer" },
                        "skipped": { "type": "integer" }
                    }
                },
                "security_status": {
                    "type": "string",
                    "enum": ["CLEAN", "WARNINGS", "VULNERABILITIES", "NOT_RUN"],
                    "description": "Security scan status"
                },
                "issues": {
                    "type": "array",
                    "description": "List of issues found",
                    "items": {
                        "type": "object",
                        "required": ["severity", "title", "description"],
                        "properties": {
                            "severity": {
                                "type": "string",
                                "enum": ["HIGH", "MEDIUM", "LOW"]
                            },
                            "title": { "type": "string" },
                            "description": { "type": "string" },
                            "file_path": { "type": "string" },
                            "line_number": { "type": "integer" },
                            "recommendation": { "type": "string" }
                        }
                    }
                },
                "verdict": {
                    "type": "string",
                    "enum": ["APPROVED", "NEEDS_CHANGES", "MAJOR_ISSUES"],
                    "description": "Review verdict"
                },
                "next_steps": {
                    "type": "string",
                    "description": "Suggested next steps"
                }
            }
        }),
    }
}

/// Execute the create_review tool
pub fn execute_create_review(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    validate_change_id(&change_id)?;

    let iteration = args
        .get("iteration")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    // Parse test results
    let test_results = if let Some(tr) = args.get("test_results") {
        TestResults {
            status: tr
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string(),
            total: tr.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            passed: tr.get("passed").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            failed: tr.get("failed").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            skipped: tr.get("skipped").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        }
    } else {
        TestResults {
            status: "UNKNOWN".to_string(),
            ..Default::default()
        }
    };

    let security_status = get_optional_string(args, "security_status")
        .unwrap_or_else(|| "NOT_RUN".to_string());

    // Parse issues
    let issues_array = get_required_array(args, "issues")?;
    let mut issues = Vec::new();

    for issue_val in issues_array {
        let severity_str = issue_val
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("MEDIUM");
        let severity = match severity_str {
            "HIGH" => Severity::High,
            "LOW" => Severity::Low,
            _ => Severity::Medium,
        };

        let title = issue_val
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Issue")
            .to_string();

        let description = issue_val
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let file_path = issue_val.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string());
        let line_number = issue_val.get("line_number").and_then(|v| v.as_u64()).map(|n| n as u32);
        let recommendation = issue_val.get("recommendation").and_then(|v| v.as_str()).map(|s| s.to_string());

        issues.push(ReviewIssue {
            severity,
            title,
            description,
            file_path,
            line_number,
            recommendation,
        });
    }

    // Parse verdict
    let verdict_str = get_required_string(args, "verdict")?;
    let verdict = match verdict_str.as_str() {
        "APPROVED" => ReviewVerdict::Approved,
        "MAJOR_ISSUES" => ReviewVerdict::MajorIssues,
        _ => ReviewVerdict::NeedsChanges,
    };

    let next_steps = get_optional_string(args, "next_steps");

    let input = CreateReviewInput {
        change_id,
        iteration,
        test_results,
        security_status,
        issues,
        verdict,
        next_steps,
    };

    crate::services::implementation_service::create_review(input, project_root)
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_change_id_valid() {
        assert!(validate_change_id("test-change").is_ok());
        assert!(validate_change_id("feature-123").is_ok());
        assert!(validate_change_id("fix-bug-42").is_ok());
    }

    #[test]
    fn test_validate_change_id_invalid() {
        assert!(validate_change_id("../etc/passwd").is_err());
        assert!(validate_change_id("/absolute/path").is_err());
        assert!(validate_change_id("Test-Change").is_err()); // uppercase
        assert!(validate_change_id("test_change").is_err()); // underscore
        assert!(validate_change_id("test..change").is_err()); // double dot
    }

    #[test]
    fn test_read_all_requirements_basic() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory structure
        let change_dir = project_root.join("agentd/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Create proposal.md
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nThis is a test proposal.",
        )
        .unwrap();

        // Create tasks.md
        std::fs::write(
            change_dir.join("tasks.md"),
            "# Tasks\n\n- Task 1\n- Task 2",
        )
        .unwrap();

        // Create specs
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("feature-spec.md"),
            "# Feature Spec\n\nRequirements here.",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute_read_all_requirements(&args, project_root).unwrap();

        assert!(result.contains("# Requirements for Change: test-change"));
        assert!(result.contains("## Proposal"));
        assert!(result.contains("This is a test proposal"));
        assert!(result.contains("## Tasks"));
        assert!(result.contains("Task 1"));
        assert!(result.contains("## Specifications"));
        assert!(result.contains("### Spec: feature-spec"));
        assert!(result.contains("**Total**: 1 proposal, 1 tasks file, 1 specification(s)"));
    }

    #[test]
    fn test_read_all_requirements_no_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory without specs
        let change_dir = project_root.join("agentd/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
        std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute_read_all_requirements(&args, project_root).unwrap();

        assert!(result.contains("**Total**: 1 proposal, 1 tasks file, 0 specification(s)"));
    }

    #[test]
    fn test_read_all_requirements_missing_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory without proposal
        let change_dir = project_root.join("agentd/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute_read_all_requirements(&args, project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("proposal.md not found"));
    }

    #[test]
    fn test_read_all_requirements_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "nonexistent"
        });

        let result = execute_read_all_requirements(&args, project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_is_git_repo() {
        // This test will pass or fail depending on whether we're in a git repo
        // Just verify it doesn't panic
        let _ = is_git_repo();
    }

    #[test]
    fn test_git_helpers_in_git_repo() {
        // Only run if we're in a git repo
        if is_git_repo() {
            let branch = get_current_branch();
            assert!(branch.is_ok());

            let status = run_git_command(&["status", "--short"]);
            assert!(status.is_ok());
        }
    }
}
