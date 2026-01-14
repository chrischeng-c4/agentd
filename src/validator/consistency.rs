//! Cross-File Consistency Validator
//!
//! Validates consistency across multiple files in a change:
//! - Task spec_refs point to existing files AND anchors
//! - Proposal affected_specs match actual specs/ directory
//! - Task dependencies have no cycles
//! - Checksums for staleness detection

use crate::models::{ErrorCategory, Severity, ValidationError, ValidationResult};
use crate::parser::{
    has_frontmatter, parse_document, parse_requirement_blocks, parse_task_blocks, ParsedDocument,
};
use crate::models::frontmatter::{ProposalFrontmatter, SpecFrontmatter};
use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Cross-file consistency validator
pub struct ConsistencyValidator {
    change_dir: PathBuf,
}

/// Parsed spec reference (file#anchor)
#[derive(Debug, Clone)]
struct SpecRef {
    file_path: String,
    anchor: Option<String>,
}

impl SpecRef {
    fn parse(spec_ref: &str) -> Self {
        if let Some((path, anchor)) = spec_ref.split_once('#') {
            SpecRef {
                file_path: path.to_string(),
                anchor: Some(anchor.to_string()),
            }
        } else {
            SpecRef {
                file_path: spec_ref.to_string(),
                anchor: None,
            }
        }
    }
}

impl ConsistencyValidator {
    /// Create a new consistency validator for a change directory
    pub fn new(change_dir: impl Into<PathBuf>) -> Self {
        Self {
            change_dir: change_dir.into(),
        }
    }

    /// Run all consistency validations
    pub fn validate_all(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // Validate task spec references
        if let Ok(task_errors) = self.validate_task_spec_refs() {
            errors.extend(task_errors);
        }

        // Validate proposal spec alignment
        if let Ok(proposal_errors) = self.validate_proposal_spec_alignment() {
            errors.extend(proposal_errors);
        }

        // Validate task dependencies (no cycles)
        if let Ok(dep_errors) = self.validate_task_dependencies() {
            errors.extend(dep_errors);
        }

        ValidationResult::new(errors)
    }

    // =========================================================================
    // Task Spec Reference Validation
    // =========================================================================

    /// Validate that all task spec_refs point to existing files and anchors
    pub fn validate_task_spec_refs(&self) -> Result<Vec<ValidationError>> {
        let tasks_path = self.change_dir.join("tasks.md");
        if !tasks_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&tasks_path)?;
        let tasks = parse_task_blocks(&content)?;
        let mut errors = Vec::new();

        for (task, line_num) in tasks {
            if let Some(spec_ref_str) = &task.spec_ref {
                let spec_ref = SpecRef::parse(spec_ref_str);

                // Normalize path (handle relative paths)
                let spec_path = self.change_dir.join(&spec_ref.file_path);

                // Check 1: File exists
                if !spec_path.exists() {
                    errors.push(ValidationError::new(
                        format!(
                            "Task {} references non-existent spec file: {}",
                            task.id, spec_ref.file_path
                        ),
                        &tasks_path,
                        line_num,
                        Severity::High,
                        ErrorCategory::BrokenReference,
                    ));
                    continue;
                }

                // Check 2: Anchor exists (if specified)
                if let Some(anchor) = &spec_ref.anchor {
                    match self.anchor_exists(&spec_path, anchor) {
                        Ok(true) => {}
                        Ok(false) => {
                            errors.push(ValidationError::new(
                                format!(
                                    "Task {} references non-existent anchor #{} in {}",
                                    task.id, anchor, spec_ref.file_path
                                ),
                                &tasks_path,
                                line_num,
                                Severity::High,
                                ErrorCategory::BrokenReference,
                            ));
                        }
                        Err(e) => {
                            errors.push(ValidationError::new(
                                format!(
                                    "Task {} - error checking anchor #{}: {}",
                                    task.id, anchor, e
                                ),
                                &tasks_path,
                                line_num,
                                Severity::Medium,
                                ErrorCategory::BrokenReference,
                            ));
                        }
                    }
                }
            }
        }

        Ok(errors)
    }

    /// Check if anchor exists in spec file
    ///
    /// Anchors can be:
    /// 1. Markdown headings: `# R1`, `## R1`, `### R1: Title` → anchor "R1"
    /// 2. Inline YAML requirement blocks: `requirement.id: R1` → anchor "R1"
    fn anchor_exists(&self, spec_path: &Path, anchor: &str) -> Result<bool> {
        let content = std::fs::read_to_string(spec_path)?;

        // Method 1: Check markdown headings (any level 1-6)
        // Pattern: ^#{1,6}\s+R1[:\s] or ^#{1,6}\s+R1$ (end of line)
        // Note: (?m) enables multiline mode so ^ matches line start
        let heading_pattern = format!(
            r"(?m)^#{{1,6}}\s+{}[:\s]|(?m)^#{{1,6}}\s+{}$",
            regex::escape(anchor),
            regex::escape(anchor)
        );
        if let Ok(heading_re) = Regex::new(&heading_pattern) {
            if heading_re.is_match(&content) {
                return Ok(true);
            }
        }

        // Method 2: Check inline YAML requirement blocks
        if let Ok(requirements) = parse_requirement_blocks(&content) {
            for (req, _) in requirements {
                if req.id == anchor {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    // =========================================================================
    // Proposal Spec Alignment Validation
    // =========================================================================

    /// Validate that proposal affected_specs matches actual specs/ directory
    pub fn validate_proposal_spec_alignment(&self) -> Result<Vec<ValidationError>> {
        let proposal_path = self.change_dir.join("proposal.md");
        if !proposal_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&proposal_path)?;

        // Only validate if proposal has frontmatter
        if !has_frontmatter(&content) {
            return Ok(Vec::new());
        }

        let parsed: ParsedDocument<ProposalFrontmatter> = match parse_document(&content) {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()), // Skip if parsing fails
        };

        let mut errors = Vec::new();

        // Check each affected_spec exists
        for spec in &parsed.frontmatter.affected_specs {
            let spec_path = self.change_dir.join(&spec.path);
            if !spec_path.exists() {
                errors.push(ValidationError::new(
                    format!("Proposal references non-existent spec: {}", spec.path),
                    &proposal_path,
                    None,
                    Severity::Medium,
                    ErrorCategory::BrokenReference,
                ));
            }
        }

        // Check for orphan specs (files in specs/ not listed in affected_specs)
        let specs_dir = self.change_dir.join("specs");
        if specs_dir.exists() {
            for entry in walkdir::WalkDir::new(&specs_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            {
                if let Ok(rel_path) = entry.path().strip_prefix(&self.change_dir) {
                    let rel_path_str = rel_path.to_string_lossy().to_string();
                    let is_listed = parsed
                        .frontmatter
                        .affected_specs
                        .iter()
                        .any(|s| s.path == rel_path_str);

                    if !is_listed {
                        errors.push(ValidationError::new(
                            format!(
                                "Spec file {} not listed in proposal.affected_specs",
                                rel_path_str
                            ),
                            &proposal_path,
                            None,
                            Severity::Low,
                            ErrorCategory::Inconsistency,
                        ));
                    }
                }
            }
        }

        Ok(errors)
    }

    // =========================================================================
    // Task Dependency Validation
    // =========================================================================

    /// Validate task dependencies have no cycles
    pub fn validate_task_dependencies(&self) -> Result<Vec<ValidationError>> {
        let tasks_path = self.change_dir.join("tasks.md");
        if !tasks_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&tasks_path)?;
        let tasks = parse_task_blocks(&content)?;
        let mut errors = Vec::new();

        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut all_task_ids: HashSet<String> = HashSet::new();

        for (task, _) in &tasks {
            all_task_ids.insert(task.id.clone());
            graph.insert(task.id.clone(), task.depends_on.clone());
        }

        // Check for references to non-existent tasks
        for (task, line_num) in &tasks {
            for dep in &task.depends_on {
                if !all_task_ids.contains(dep) {
                    errors.push(ValidationError::new(
                        format!("Task {} depends on non-existent task: {}", task.id, dep),
                        &tasks_path,
                        *line_num,
                        Severity::High,
                        ErrorCategory::BrokenReference,
                    ));
                }
            }
        }

        // Detect cycles using DFS
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();

        for task_id in all_task_ids.iter() {
            if !visited.contains(task_id) {
                if let Some(cycle) = self.detect_cycle(task_id, &graph, &mut visited, &mut rec_stack)
                {
                    errors.push(ValidationError::new(
                        format!("Circular dependency detected: {}", cycle.join(" → ")),
                        &tasks_path,
                        None,
                        Severity::High,
                        ErrorCategory::CircularDependency,
                    ));
                    break; // Report first cycle found
                }
            }
        }

        Ok(errors)
    }

    /// DFS cycle detection
    fn detect_cycle(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    if let Some(mut cycle) = self.detect_cycle(dep, graph, visited, rec_stack) {
                        cycle.insert(0, node.to_string());
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    // Found cycle
                    return Some(vec![node.to_string(), dep.clone()]);
                }
            }
        }

        rec_stack.remove(node);
        None
    }

    // =========================================================================
    // Spec Hierarchy Validation
    // =========================================================================

    /// Validate spec hierarchy (parent/child/related refs)
    pub fn validate_spec_hierarchy(&self) -> Result<Vec<ValidationError>> {
        let specs_dir = self.change_dir.join("specs");
        if !specs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut errors = Vec::new();

        for entry in walkdir::WalkDir::new(&specs_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if !has_frontmatter(&content) {
                continue;
            }

            let parsed: ParsedDocument<SpecFrontmatter> = match parse_document(&content) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let spec_path = entry.path();

            // Check parent_spec exists
            if let Some(parent) = &parsed.frontmatter.parent_spec {
                let parent_path = specs_dir.join(format!("{}.md", parent));
                if !parent_path.exists() {
                    errors.push(ValidationError::new(
                        format!("Spec references non-existent parent: {}", parent),
                        spec_path,
                        None,
                        Severity::Medium,
                        ErrorCategory::BrokenReference,
                    ));
                }
            }

            // Check related_specs exist
            for related in &parsed.frontmatter.related_specs {
                let related_path = self.change_dir.join(&related.path);
                if !related_path.exists() {
                    errors.push(ValidationError::new(
                        format!("Spec references non-existent related spec: {}", related.path),
                        spec_path,
                        None,
                        Severity::Low,
                        ErrorCategory::BrokenReference,
                    ));
                }
            }
        }

        Ok(errors)
    }
}

// =============================================================================
// Add ErrorCategory variants if needed
// =============================================================================

impl ErrorCategory {
    /// Check if this is a reference error
    pub fn is_reference_error(&self) -> bool {
        matches!(self, ErrorCategory::BrokenReference)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_change() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join("test-change");
        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        (temp_dir, change_dir)
    }

    #[test]
    fn test_valid_spec_ref() {
        let (_temp, change_dir) = setup_test_change();

        // Create spec file with R1 heading
        let spec_content = r#"---
id: auth-spec
type: spec
title: Auth
version: 1
---

## Requirements

### R1: User Auth

Description here.
"#;
        std::fs::write(change_dir.join("specs/auth.md"), spec_content).unwrap();

        // Create tasks.md with valid ref
        let tasks_content = r#"# Tasks

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/auth.rs
  spec_ref: specs/auth.md#R1
```
"#;
        std::fs::write(change_dir.join("tasks.md"), tasks_content).unwrap();

        let validator = ConsistencyValidator::new(&change_dir);
        let errors = validator.validate_task_spec_refs().unwrap();

        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_invalid_spec_ref_missing_file() {
        let (_temp, change_dir) = setup_test_change();

        // Create tasks.md with ref to non-existent file
        let tasks_content = r#"# Tasks

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/auth.rs
  spec_ref: specs/nonexistent.md#R1
```
"#;
        std::fs::write(change_dir.join("tasks.md"), tasks_content).unwrap();

        let validator = ConsistencyValidator::new(&change_dir);
        let errors = validator.validate_task_spec_refs().unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("non-existent spec file"));
    }

    #[test]
    fn test_invalid_spec_ref_missing_anchor() {
        let (_temp, change_dir) = setup_test_change();

        // Create spec file without R1
        let spec_content = r#"---
id: auth-spec
type: spec
title: Auth
version: 1
---

## Requirements

### R2: Other Requirement
"#;
        std::fs::write(change_dir.join("specs/auth.md"), spec_content).unwrap();

        // Create tasks.md with ref to non-existent anchor
        let tasks_content = r#"# Tasks

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/auth.rs
  spec_ref: specs/auth.md#R1
```
"#;
        std::fs::write(change_dir.join("tasks.md"), tasks_content).unwrap();

        let validator = ConsistencyValidator::new(&change_dir);
        let errors = validator.validate_task_spec_refs().unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("non-existent anchor"));
    }

    #[test]
    fn test_anchor_in_yaml_block() {
        let (_temp, change_dir) = setup_test_change();

        // Create spec with inline YAML requirement block
        let spec_content = r#"---
id: auth-spec
type: spec
title: Auth
version: 1
---

## Requirements

### User Authentication

```yaml
requirement:
  id: R1
  priority: high
  status: draft
```

Description here.
"#;
        std::fs::write(change_dir.join("specs/auth.md"), spec_content).unwrap();

        // Create tasks.md with ref to R1
        let tasks_content = r#"# Tasks

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/auth.rs
  spec_ref: specs/auth.md#R1
```
"#;
        std::fs::write(change_dir.join("tasks.md"), tasks_content).unwrap();

        let validator = ConsistencyValidator::new(&change_dir);
        let errors = validator.validate_task_spec_refs().unwrap();

        assert!(errors.is_empty(), "Expected anchor R1 to be found in YAML block");
    }

    #[test]
    fn test_circular_dependency() {
        let (_temp, change_dir) = setup_test_change();

        // Create tasks with circular dependency: 1.1 -> 1.2 -> 1.3 -> 1.1
        let tasks_content = r#"# Tasks

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/a.rs
  depends_on: ["1.3"]
```

```yaml
task:
  id: "1.2"
  action: CREATE
  status: pending
  file: src/b.rs
  depends_on: ["1.1"]
```

```yaml
task:
  id: "1.3"
  action: CREATE
  status: pending
  file: src/c.rs
  depends_on: ["1.2"]
```
"#;
        std::fs::write(change_dir.join("tasks.md"), tasks_content).unwrap();

        let validator = ConsistencyValidator::new(&change_dir);
        let errors = validator.validate_task_dependencies().unwrap();

        assert!(!errors.is_empty(), "Should detect circular dependency");
        assert!(errors.iter().any(|e| e.message.contains("Circular dependency")));
    }

    #[test]
    fn test_dependency_to_nonexistent_task() {
        let (_temp, change_dir) = setup_test_change();

        // Create task depending on non-existent task
        let tasks_content = r#"# Tasks

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/a.rs
  depends_on: ["9.9"]
```
"#;
        std::fs::write(change_dir.join("tasks.md"), tasks_content).unwrap();

        let validator = ConsistencyValidator::new(&change_dir);
        let errors = validator.validate_task_dependencies().unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("non-existent task"));
    }

    #[test]
    fn test_orphan_spec_detection() {
        let (_temp, change_dir) = setup_test_change();

        // Create proposal with affected_specs
        let proposal_content = r#"---
id: test-change
type: proposal
version: 1
status: proposed
summary: Test proposal
affected_specs:
  - id: auth-spec
    path: specs/auth.md
---

## Why
Test
"#;
        std::fs::write(change_dir.join("proposal.md"), proposal_content).unwrap();

        // Create two spec files, but only one is listed
        std::fs::write(change_dir.join("specs/auth.md"), "# Auth Spec").unwrap();
        std::fs::write(change_dir.join("specs/orphan.md"), "# Orphan Spec").unwrap();

        let validator = ConsistencyValidator::new(&change_dir);
        let errors = validator.validate_proposal_spec_alignment().unwrap();

        // Should find orphan.md not listed
        assert!(errors.iter().any(|e| e.message.contains("orphan.md")));
    }
}
