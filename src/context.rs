use crate::Result;
use colored::Colorize;
use dialoguer::Select;
use std::path::Path;
use walkdir::WalkDir;

const GEMINI_TEMPLATE: &str = include_str!("../templates/GEMINI.md");
const AGENTS_TEMPLATE: &str = include_str!("../templates/AGENTS.md");

/// Generate GEMINI.md context file for a specific change
pub fn generate_gemini_context(change_dir: &Path) -> Result<()> {
    let project_structure = scan_project_structure()?;
    let content = GEMINI_TEMPLATE.replace("{{PROJECT_STRUCTURE}}", &project_structure);

    let output_path = change_dir.join("GEMINI.md");
    std::fs::write(&output_path, content)?;

    Ok(())
}

/// Generate AGENTS.md context file for a specific change
pub fn generate_agents_context(change_dir: &Path) -> Result<()> {
    let project_structure = scan_project_structure()?;
    let content = AGENTS_TEMPLATE.replace("{{PROJECT_STRUCTURE}}", &project_structure);

    let output_path = change_dir.join("AGENTS.md");
    std::fs::write(&output_path, content)?;

    Ok(())
}

/// Scan project structure and generate a tree representation
fn scan_project_structure() -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let mut output = String::from("```\n");

    // Scan important directories
    let important_dirs = vec!["src", "specter/specs", "specter/changes"];

    for dir in important_dirs {
        let path = current_dir.join(dir);
        if path.exists() {
            output.push_str(&format!("{}:\n", dir));
            output.push_str(&scan_directory(&path, 2)?);
            output.push('\n');
        }
    }

    output.push_str("```");
    Ok(output)
}

/// Recursively scan a directory with depth limit
fn scan_directory(path: &Path, max_depth: usize) -> Result<String> {
    let mut output = String::new();
    let entries: Vec<_> = WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden files and common ignore patterns
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "dist"
        })
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let depth = entry.depth();
        if depth == 0 {
            continue;
        }

        let indent = "  ".repeat(depth);
        let name = entry.file_name().to_string_lossy();

        if entry.file_type().is_dir() {
            output.push_str(&format!("{}{}/\n", indent, name));
        } else {
            output.push_str(&format!("{}{}\n", indent, name));
        }
    }

    Ok(output)
}

/// Create proposal skeleton files with structure but no content
/// This guides Gemini to fill in the right format while reducing token usage
pub fn create_proposal_skeleton(change_dir: &Path, change_id: &str) -> Result<()> {
    // Create proposal.md skeleton
    let proposal_skeleton = format!(
        r#"# Change: {change_id}

## Summary
[Brief 1-2 sentence description]

## Why
[Problem statement and motivation]

## What Changes
[Bullet points of what will be added/modified/removed]

## Impact
- Affected specs: [capabilities]
- Affected code: [files/systems]
- Breaking changes: [Yes/No]
"#,
        change_id = change_id
    );
    std::fs::write(change_dir.join("proposal.md"), proposal_skeleton)?;

    // Create tasks.md skeleton
    let tasks_skeleton = r#"# Tasks

## 1. Implementation
- [ ] 1.1 [Task description]
- [ ] 1.2 [Task description]

## 2. Testing
- [ ] 2.1 Unit tests: [What to test]
- [ ] 2.2 Integration tests: [What to test]

## 3. Documentation
- [ ] 3.1 Update specs: [Which specs]
- [ ] 3.2 Update README: [If needed]
"#;
    std::fs::write(change_dir.join("tasks.md"), tasks_skeleton)?;

    // Create diagrams.md skeleton
    let diagrams_skeleton = r#"# Architecture Diagrams

## State Diagram
```mermaid
stateDiagram-v2
    [Add state transitions here]
```

## Flow Diagram
```mermaid
flowchart TD
    [Add process flow here]
```

## Sequence Diagram
```mermaid
sequenceDiagram
    [Add interactions here]
```
"#;
    std::fs::write(change_dir.join("diagrams.md"), diagrams_skeleton)?;

    // Create specs directory with a skeleton file
    let specs_dir = change_dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    let spec_skeleton = r#"# Spec Delta

## ADDED Requirements
### Requirement: [Name]
The system SHALL [requirement description].

#### Scenario: Success case
- **WHEN** [trigger condition]
- **THEN** [expected behavior]

#### Scenario: Error case
- **WHEN** [error condition]
- **THEN** [error handling]

## MODIFIED Requirements
[If modifying existing requirements, include FULL updated requirement text]

## REMOVED Requirements
[If removing requirements, explain why and migration path]
"#;
    std::fs::write(specs_dir.join("_skeleton.md"), spec_skeleton)?;

    Ok(())
}

/// Create challenge skeleton file to guide Codex review
/// This provides structure for consistent challenge reports
pub fn create_challenge_skeleton(change_dir: &Path, change_id: &str) -> Result<()> {
    let challenge_skeleton = format!(
        r#"# Challenge Report: {change_id}

## Summary
[Overall assessment of the proposal quality and readiness]

## Internal Consistency Issues
[Check if proposal files are consistent with each other]
[Examples: Does proposal.md match tasks.md? Do diagrams.md match descriptions? Do specs/ align with Impact?]
[These are HIGH priority - must fix before implementation]

### Issue: [Title]
- **Severity**: High
- **Category**: Completeness | Consistency
- **Description**: [What's inconsistent]
- **Location**: [Which files/sections]
- **Recommendation**: [How to fix]

## Code Alignment Issues
[Check if proposal aligns with existing codebase]
[Examples: Do file paths exist? Do APIs exist? Does architecture match patterns?]
[IMPORTANT: If proposal mentions refactoring or BREAKING changes, deviations are EXPECTED]

### Issue: [Title]
- **Severity**: Medium | Low
- **Category**: Conflict | Other
- **Description**: [What differs from existing code]
- **Location**: [File paths or components]
- **Note**: [Is this intentional? Check proposal.md for refactor mentions]
- **Recommendation**: [Clarify or confirm intention]

## Quality Suggestions
[Missing tests, error handling, edge cases, documentation]
[These are LOW priority - nice to have improvements]

### Issue: [Title]
- **Severity**: Low
- **Category**: Completeness | Other
- **Description**: [What could be improved]
- **Recommendation**: [Suggested enhancement]

## Verdict
- [ ] APPROVED - Ready for implementation
- [ ] NEEDS_REVISION - Address issues above (specify which severity levels)
- [ ] REJECTED - Fundamental problems, needs rethinking

**Next Steps**: [What should be done based on verdict]
"#,
        change_id = change_id
    );

    std::fs::write(change_dir.join("CHALLENGE.md"), challenge_skeleton)?;
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
