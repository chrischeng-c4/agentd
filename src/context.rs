use crate::Result;
use colored::Colorize;
use dialoguer::Select;
use std::path::Path;

// Implementation phase skeleton (inline)
const REVIEW_SKELETON: &str = r#"# Code Review Report: {{change_id}}

**Iteration**: {{iteration}}

## Summary
[Overall assessment: code quality, test results, security posture]

## Test Results
**Overall Status**: PASS | FAIL | PARTIAL

### Test Summary
- Total tests: X
- Passed: X
- Failed: X
- Skipped: X
- Coverage: X%

### Failed Tests (if any)
- `test_name`: [Error message]

## Security Scan Results
**Status**: CLEAN | WARNINGS | VULNERABILITIES

### cargo audit (Dependency Vulnerabilities)
- [List vulnerabilities or "No vulnerabilities found"]

### semgrep (Code Pattern Scan)
- [List security issues or "No issues found"]

## Best Practices Issues
[HIGH priority - must fix]

### Issue: [Title]
- **Severity**: High
- **Category**: Security | Performance | Style
- **File**: path/to/file.rs:123
- **Description**: [What's wrong]
- **Recommendation**: [How to fix]

## Requirement Compliance Issues
[HIGH priority - must fix]

### Issue: [Title]
- **Severity**: High
- **Category**: Missing Feature | Wrong Behavior
- **Requirement**: [Which spec/task]
- **Description**: [What's missing or wrong]
- **Recommendation**: [How to fix]

## Consistency Issues
[MEDIUM priority - should fix]

### Issue: [Title]
- **Severity**: Medium
- **Category**: Style | Architecture | Naming
- **Location**: path/to/file
- **Description**: [How it differs from codebase patterns]
- **Recommendation**: [How to align]

## Test Quality Issues
[MEDIUM priority - should fix]

### Issue: [Title]
- **Severity**: Medium
- **Category**: Coverage | Edge Case | Scenario
- **Description**: [What's missing in tests]
- **Recommendation**: [What to add]

## Verdict
- [ ] APPROVED - Ready for merge (all tests pass, no HIGH issues)
- [ ] NEEDS_CHANGES - Address issues above (specify which)
- [ ] MAJOR_ISSUES - Fundamental problems (failing tests or critical security)

**Next Steps**: [What should be done]
"#;

// Archive phase skeleton (inline)
const ARCHIVE_REVIEW_SKELETON: &str = r#"# Archive Review Report: {{change_id}}

**Iteration**: {{iteration}}

## Summary
[Overall assessment: merge quality, spec consistency, completeness]

## Merge Quality

### Spec Integration
- **Status**: CLEAN | ISSUES
- [Assessment of how well change specs were merged into main specs]

### Content Preservation
- **Requirements preserved**: Yes | No (list missing)
- **Scenarios preserved**: Yes | No (list missing)
- **Diagrams preserved**: Yes | No (list missing)

## Issues Found

### Issue: [Title]
- **Severity**: High | Medium | Low
- **Category**: Missing Content | Duplicate Content | Format Error | Inconsistency
- **File**: agentd/specs/[file.md]
- **Description**: [What's wrong]
- **Recommendation**: [How to fix]

## CHANGELOG Quality
- **Entry present**: Yes | No
- **Description accurate**: Yes | No
- **Format correct**: Yes | No

## Verdict
- [ ] APPROVED - Merge quality acceptable, ready for archive
- [ ] NEEDS_FIX - Address issues above (fixable automatically)
- [ ] REJECTED - Fundamental problems (require manual intervention)

**Next Steps**: [What should be done]
"#;


/// Create REVIEW.md skeleton for code review process
pub fn create_review_skeleton(change_dir: &Path, change_id: &str, iteration: u32) -> Result<()> {
    let content = REVIEW_SKELETON
        .replace("{{change_id}}", change_id)
        .replace("{{iteration}}", &iteration.to_string());
    std::fs::write(change_dir.join("REVIEW.md"), content)?;
    Ok(())
}

/// Create ARCHIVE_REVIEW.md skeleton for archive quality review
pub fn create_archive_review_skeleton(change_dir: &Path, change_id: &str, iteration: u32) -> Result<()> {
    let content = ARCHIVE_REVIEW_SKELETON
        .replace("{{change_id}}", change_id)
        .replace("{{iteration}}", &iteration.to_string());
    std::fs::write(change_dir.join("ARCHIVE_REVIEW.md"), content)?;
    Ok(())
}

/// Clean up generated context files when archiving
pub fn cleanup_context_files(change_dir: &Path) -> Result<()> {
    let gemini_path = change_dir.join("GEMINI.md");
    let agents_path = change_dir.join("AGENTS.md");

    if gemini_path.exists() {
        std::fs::remove_file(gemini_path)?;
    }

    if agents_path.exists() {
        std::fs::remove_file(agents_path)?;
    }

    Ok(())
}

/// Conflict resolution strategy chosen by user
enum ConflictResolution {
    UseSuggested(String),
    Abort,
}

/// Resolves change-id conflicts by finding next available ID or prompting user
///
/// This function is called early in the proposal workflow (before calling LLMs)
/// to handle the case when a change directory already exists.
///
/// In interactive mode: Presents user with 3 options
/// In non-interactive mode: Auto-uses the suggested ID
pub fn resolve_change_id_conflict(change_id: &str, changes_dir: &Path) -> Result<String> {
    let change_dir = changes_dir.join(change_id);

    // No conflict - use original ID
    if !change_dir.exists() {
        return Ok(change_id.to_string());
    }

    // If directory only has clarifications.md (no STATE.yaml or proposal.md),
    // it's the expected state after clarification phase - not a conflict
    let state_exists = change_dir.join("STATE.yaml").exists();
    let proposal_exists = change_dir.join("proposal.md").exists();
    if !state_exists && !proposal_exists {
        // This is a fresh change with only clarifications - continue with same ID
        return Ok(change_id.to_string());
    }

    // Conflict detected - find next available ID
    let suggested_id = find_next_available_id(change_id, changes_dir);
    let similar_changes = list_similar_changes(change_id, changes_dir);

    println!();
    println!("{}", "⚠️  Change already exists".yellow().bold());
    println!();

    // List similar existing changes
    if !similar_changes.is_empty() {
        println!("{}", "Existing changes:".bright_black());
        for change in &similar_changes {
            // Try to get creation time
            let change_path = changes_dir.join(change);
            if let Ok(metadata) = std::fs::metadata(&change_path) {
                if let Ok(created) = metadata.created() {
                    let datetime: chrono::DateTime<chrono::Local> = created.into();
                    println!(
                        "  • {}/ {}",
                        change,
                        format!("(created {})", datetime.format("%Y-%m-%d")).bright_black()
                    );
                } else {
                    println!("  • {}/", change);
                }
            } else {
                println!("  • {}/", change);
            }
        }
        println!();
    }

    // Try interactive prompt
    match prompt_conflict_resolution(change_id, &suggested_id, &change_dir) {
        Ok(resolution) => match resolution {
            ConflictResolution::UseSuggested(id) => {
                println!(
                    "{}",
                    format!("Using new ID: '{}'", id).green()
                );
                println!();
                Ok(id)
            }
            ConflictResolution::Abort => {
                anyhow::bail!("Operation aborted by user");
            }
        },
        Err(_) => {
            // Non-interactive mode or terminal not available
            // Auto-use suggested ID with warning
            println!(
                "{}",
                format!(
                    "(non-interactive mode: using new ID '{}')",
                    suggested_id
                )
                .bright_black()
            );
            println!();
            Ok(suggested_id)
        }
    }
}

/// Find next available change ID with numeric suffix
///
/// Given a base ID like "test-oauth", finds the next available numeric suffix:
/// - test-oauth exists -> test-oauth-2
/// - test-oauth, test-oauth-2 exist -> test-oauth-3
/// - test-oauth, test-oauth-5 exist -> test-oauth-6 (finds highest + 1)
fn find_next_available_id(base_id: &str, changes_dir: &Path) -> String {
    let mut highest = 1;

    // First, scan for any existing numbered versions to find the highest
    if let Ok(entries) = std::fs::read_dir(changes_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                // Check if this matches the pattern base_id-N
                if let Some(suffix) = name.strip_prefix(&format!("{}-", base_id)) {
                    if let Ok(num) = suffix.parse::<u32>() {
                        highest = highest.max(num);
                    }
                }
            }
        }
    }

    // Start from highest + 1
    let mut counter = highest + 1;

    // Find next available (in case there are gaps)
    loop {
        let candidate = format!("{}-{}", base_id, counter);
        if !changes_dir.join(&candidate).exists() {
            return candidate;
        }
        counter += 1;
    }
}

/// List existing changes with similar names
///
/// Returns a sorted list of change directories that start with the base_id.
/// For example, with base_id="test-oauth", returns:
/// ["test-oauth", "test-oauth-2", "test-oauth-3"]
fn list_similar_changes(base_id: &str, changes_dir: &Path) -> Vec<String> {
    let mut similar = Vec::new();

    if let Ok(entries) = std::fs::read_dir(changes_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Match exact or numbered pattern
                    if name == base_id || name.starts_with(&format!("{}-", base_id)) {
                        similar.push(name.to_string());
                    }
                }
            }
        }
    }

    similar.sort();
    similar
}

/// Interactive prompt for conflict resolution
///
/// Presents user with 2 options:
/// 1. Use suggested ID (recommended)
/// 2. Abort and manually handle
///
/// Returns Err if terminal is not available (non-interactive mode)
fn prompt_conflict_resolution(
    _original_id: &str,
    suggested_id: &str,
    _existing_path: &Path,
) -> Result<ConflictResolution> {
    let options = vec![
        format!("Use new ID '{}' (recommended)", suggested_id),
        "Abort (manually delete or use different ID)".to_string(),
    ];

    println!("{}", "What would you like to do?".cyan());

    let selection = Select::new()
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!("Terminal not available: {}", e))?;

    match selection {
        0 => Ok(ConflictResolution::UseSuggested(suggested_id.to_string())),
        1 => Ok(ConflictResolution::Abort),
        _ => Ok(ConflictResolution::Abort),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_review_skeleton() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();

        create_review_skeleton(change_dir, "test-change", 1).unwrap();

        let content = fs::read_to_string(change_dir.join("REVIEW.md")).unwrap();
        assert!(content.contains("# Code Review Report: test-change"));
        assert!(content.contains("**Iteration**: 1"));
    }

    #[test]
    fn test_create_archive_review_skeleton() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();

        create_archive_review_skeleton(change_dir, "my-change", 2).unwrap();

        let content = fs::read_to_string(change_dir.join("ARCHIVE_REVIEW.md")).unwrap();
        assert!(content.contains("# Archive Review Report: my-change"));
        assert!(content.contains("**Iteration**: 2"));
    }
}
