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
    let important_dirs = vec!["src", "agentd/specs", "agentd/changes"];

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

    // Create tasks.md skeleton (Ticket format)
    let tasks_skeleton = r#"# Tasks

<!--
Each task is a dev ticket derived from specs.
NO actual code - just file paths, actions, and references.
-->

## 1. Data Layer
- [ ] 1.1 [Task title]
  - File: `path/to/file.rs` (CREATE|MODIFY|DELETE)
  - Spec: `specs/[name].md#[section]`
  - Do: [What to implement - not how]

## 2. Logic Layer
- [ ] 2.1 [Task title]
  - File: `path/to/file.rs` (CREATE|MODIFY)
  - Spec: `specs/[name].md#[section]`
  - Do: [What to implement]
  - Depends: 1.1

## 3. Integration
- [ ] 3.1 [Task title]
  - File: `path/to/file.rs` (MODIFY)
  - Do: [What to integrate]
  - Depends: 2.1

## 4. Testing
- [ ] 4.1 [Test task title]
  - File: `path/to/test.rs` (CREATE)
  - Verify: `specs/[name].md#acceptance-criteria`
  - Depends: [relevant tasks]
"#;
    std::fs::write(change_dir.join("tasks.md"), tasks_skeleton)?;

    // Create specs directory with a skeleton file
    let specs_dir = change_dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    // Spec skeleton: TD + AC format with diagrams (NO actual code)
    let spec_skeleton = r#"# Spec: [Feature Name]

<!--
Technical Design + Acceptance Criteria.
Use abstraction tools: Mermaid, JSON Schema, OpenAPI, Pseudo code.
NO actual implementation code.
-->

## Overview
[Brief description of what this spec covers]

## Flow
```mermaid
sequenceDiagram
    participant U as User
    participant S as System
    U->>S: [action]
    S-->>U: [response]
```

## State (if applicable)
```mermaid
stateDiagram-v2
    [*] --> State1
    State1 --> State2: event
    State2 --> [*]
```

## Data Model
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "field1": { "type": "string", "description": "..." },
    "field2": { "type": "integer" }
  },
  "required": ["field1"]
}
```

## Interfaces
```
FUNCTION function_name(param1: type, param2: type) -> ResultType
  INPUT: [describe inputs]
  OUTPUT: [describe outputs]
  ERRORS: [possible error conditions]

FUNCTION another_function() -> void
  SIDE_EFFECTS: [what it modifies]
```

## Acceptance Criteria
- WHEN [trigger condition] THEN [expected behavior]
- WHEN [error condition] THEN [error handling]
- WHEN [edge case] THEN [expected behavior]
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
[Examples: Does proposal.md match tasks.md? Do Mermaid diagrams in specs/ match descriptions? Do task spec refs exist?]
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

/// Create REVIEW.md skeleton for code review process
pub fn create_review_skeleton(change_dir: &Path, change_id: &str, iteration: u32) -> Result<()> {
    let review_skeleton = format!(
        r#"# Code Review Report: {change_id}

**Iteration**: {iteration}

## Summary
[Overall assessment: code quality, test results, security posture]

## Test Results
**Overall Status**: ✅ PASS | ❌ FAIL | ⚠️ PARTIAL

### Test Summary
- Total tests: X
- Passed: X
- Failed: X
- Skipped: X
- Coverage: X%

### Failed Tests (if any)
- `test_name`: [Error message]
- `test_name_2`: [Error message]

## Security Scan Results
**Status**: ✅ CLEAN | ⚠️ WARNINGS | ❌ VULNERABILITIES

### cargo audit (Dependency Vulnerabilities)
- [List vulnerabilities or "No vulnerabilities found"]

### semgrep (Code Pattern Scan)
- [List security issues or "No issues found"]

### Linter Security Rules
- [List warnings or "No warnings"]

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
"#,
        change_id = change_id,
        iteration = iteration
    );

    std::fs::write(change_dir.join("REVIEW.md"), review_skeleton)?;
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
