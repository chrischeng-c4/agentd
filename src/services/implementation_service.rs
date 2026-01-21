//! Implementation service - Business logic for implementation workflow
//!
//! Provides functions to read requirements and list changed files during
//! the implementation and review stages.

use crate::Result;
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

/// Run a git command and return output
fn run_git_command(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(format!("⚠️ Git command failed: {}", stderr.trim()));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Read all requirement files (proposal, tasks, specs) for a change in one call
pub fn read_all_requirements(change_id: &str, project_root: &Path) -> Result<String> {
    validate_change_id(change_id)?;

    let change_dir = project_root.join("agentd/changes").join(change_id);
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

/// List changed files with detailed statistics (additions/deletions)
pub fn list_changed_files(
    change_id: &str,
    base_branch: Option<&str>,
    filter: Option<&str>,
    _project_root: &Path,
) -> Result<String> {
    validate_change_id(change_id)?;

    let base_branch = base_branch.unwrap_or("main");

    if !is_git_repo() {
        anyhow::bail!("Not in a git repository");
    }

    let mut output = String::new();
    output.push_str(&format!("# Changed Files for: {}\n\n", change_id));

    if let Some(f) = filter {
        output.push_str(&format!("**Filter**: `{}`\n\n", f));
    }

    // Get numstat output
    let numstat = run_git_command(&["diff", "--numstat", base_branch])?;

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
        if let Some(f) = filter {
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

        let result = read_all_requirements("test-change", project_root).unwrap();

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

        let result = read_all_requirements("test-change", project_root).unwrap();

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

        let result = read_all_requirements("test-change", project_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("proposal.md not found"));
    }

    #[test]
    fn test_is_git_repo() {
        // This test will pass or fail depending on whether we're in a git repo
        // Just verify it doesn't panic
        let _ = is_git_repo();
    }
}
