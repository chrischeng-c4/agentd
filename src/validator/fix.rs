//! Auto-fixer for validation errors
//!
//! This module provides automatic fixes for common validation errors that can be
//! mechanically corrected without AI intervention.

use crate::models::{ErrorCategory, ValidationError};
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Auto-fixer for validation errors
pub struct AutoFixer {
    project_root: PathBuf,
}

/// Result of a fix attempt
#[derive(Debug)]
pub struct FixResult {
    /// Number of files modified
    pub files_modified: usize,
    /// Number of errors fixed
    pub errors_fixed: usize,
    /// Errors that could not be fixed
    pub unfixable_errors: Vec<ValidationError>,
    /// Details of what was fixed
    pub fix_details: Vec<String>,
}

impl AutoFixer {
    /// Create a new auto-fixer
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    /// Attempt to fix validation errors
    pub fn fix_errors(&self, errors: &[ValidationError]) -> Result<FixResult> {
        let mut files_modified = 0;
        let mut errors_fixed = 0;
        let mut unfixable_errors = Vec::new();
        let mut fix_details = Vec::new();

        // Group errors by file
        let mut errors_by_file: HashMap<PathBuf, Vec<&ValidationError>> = HashMap::new();
        for error in errors {
            errors_by_file
                .entry(error.file.clone())
                .or_default()
                .push(error);
        }

        // Process each file
        for (file_path, file_errors) in errors_by_file {
            let full_path = if file_path.is_absolute() {
                file_path.clone()
            } else {
                self.project_root.join(&file_path)
            };

            if !full_path.exists() {
                for error in file_errors {
                    unfixable_errors.push(error.clone());
                }
                continue;
            }

            let content = std::fs::read_to_string(&full_path)?;
            let mut modified_content = content.clone();
            let mut file_fixed = false;

            for error in file_errors {
                if !error.category.is_fixable() {
                    unfixable_errors.push(error.clone());
                    continue;
                }

                match self.apply_fix(&mut modified_content, error) {
                    Some(detail) => {
                        fix_details.push(detail);
                        errors_fixed += 1;
                        file_fixed = true;
                    }
                    None => {
                        unfixable_errors.push(error.clone());
                    }
                }
            }

            if file_fixed && modified_content != content {
                std::fs::write(&full_path, &modified_content)?;
                files_modified += 1;
            }
        }

        Ok(FixResult {
            files_modified,
            errors_fixed,
            unfixable_errors,
            fix_details,
        })
    }

    /// Apply a single fix and return a description of what was done
    fn apply_fix(&self, content: &mut String, error: &ValidationError) -> Option<String> {
        match error.category {
            ErrorCategory::MissingHeading => self.fix_missing_heading(content, error),
            ErrorCategory::MissingWhenThen => self.fix_missing_when_then(content, error),
            ErrorCategory::MissingScenario => self.fix_missing_scenario(content, error),
            _ => None,
        }
    }

    /// Check if content has Acceptance Criteria with proper WHEN/THEN
    fn has_valid_acceptance_criteria(content: &str) -> bool {
        if let Some(ac_pos) = content.find("## Acceptance Criteria") {
            let after_ac = &content[ac_pos..];
            // Check if there's at least one scenario with WHEN and THEN
            after_ac.contains("WHEN") && after_ac.contains("THEN")
        } else {
            false
        }
    }

    /// Fix missing heading by appending it to the file
    fn fix_missing_heading(&self, content: &mut String, error: &ValidationError) -> Option<String> {
        // Extract heading name from error message
        // Format: "Missing required heading: Overview"
        let heading_name = error
            .message
            .strip_prefix("Missing required heading: ")?;

        // Check if heading already exists (case-insensitive)
        let heading_lower = heading_name.to_lowercase();
        if content.to_lowercase().contains(&format!("## {}", heading_lower)) {
            return None;
        }

        // Append the missing heading
        let heading_content = match heading_name {
            "Overview" => "\n\n## Overview\n\n<!-- Brief description of this feature -->\n",
            "Acceptance Criteria" => {
                "\n\n## Acceptance Criteria\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n"
            }
            "Requirements" => "\n\n## Requirements\n\n### R1: Basic Requirement\nDescription of the requirement.\n",
            _ => return None, // Unknown heading, can't auto-fix
        };

        content.push_str(heading_content);
        Some(format!(
            "{}: Added missing '## {}' heading",
            error.file.display(),
            heading_name
        ))
    }

    /// Fix missing WHEN/THEN by adding Acceptance Criteria section with placeholder scenario
    fn fix_missing_when_then(
        &self,
        content: &mut String,
        error: &ValidationError,
    ) -> Option<String> {
        // Check if we already have valid Acceptance Criteria
        if Self::has_valid_acceptance_criteria(content) {
            return None;
        }

        // If there's no Acceptance Criteria section, add one with a placeholder scenario
        if !content.contains("## Acceptance Criteria") {
            let ac_section = "\n\n## Acceptance Criteria\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
            content.push_str(ac_section);
            return Some(format!(
                "{}: Added Acceptance Criteria with WHEN/THEN",
                error.file.display()
            ));
        }

        // There's an AC section but no WHEN/THEN - add a placeholder scenario
        if let Some(ac_pos) = content.find("## Acceptance Criteria") {
            let ac_end = ac_pos + "## Acceptance Criteria".len();
            let scenario = "\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
            content.insert_str(ac_end, scenario);
            return Some(format!(
                "{}: Added placeholder scenario with WHEN/THEN",
                error.file.display()
            ));
        }

        // Fallback: try to find scenario without WHEN/THEN
        // Error format: "Scenario 'X' is missing WHEN clause" (old format)
        let scenario_name = if error.message.contains("WHEN") {
            error
                .message
                .strip_prefix("Scenario '")?
                .split("' is missing")
                .next()?
        } else if error.message.contains("THEN") {
            error
                .message
                .strip_prefix("Scenario '")?
                .split("' is missing")
                .next()?
        } else {
            return None;
        };

        // Find the scenario in content
        let scenario_marker = format!("Scenario: {}", scenario_name);
        if let Some(pos) = content.find(&scenario_marker) {
            // Find the end of this scenario (next ### or end of file)
            let after_scenario = &content[pos..];
            let scenario_end = after_scenario
                .find("\n### ")
                .or_else(|| after_scenario.find("\n## "))
                .unwrap_or(after_scenario.len());

            let scenario_content = &content[pos..pos + scenario_end];

            // Check what's missing and add it
            let missing_when = !scenario_content.contains("WHEN");
            let missing_then = !scenario_content.contains("THEN");

            if missing_when || missing_then {
                let insert_pos = pos + scenario_end;
                let mut to_insert = String::new();

                if missing_when {
                    to_insert.push_str("\n- **WHEN** [condition]");
                }
                if missing_then {
                    to_insert.push_str("\n- **THEN** [expected result]");
                }

                content.insert_str(insert_pos, &to_insert);
                return Some(format!(
                    "{}: Added WHEN/THEN to scenario '{}'",
                    error.file.display(),
                    scenario_name
                ));
            }
        }

        None
    }

    /// Fix missing scenario by adding a placeholder
    fn fix_missing_scenario(&self, content: &mut String, error: &ValidationError) -> Option<String> {
        // Check if we already have valid Acceptance Criteria
        if Self::has_valid_acceptance_criteria(content) {
            return None;
        }

        // If there's no Acceptance Criteria section, add one with a placeholder scenario
        if !content.contains("## Acceptance Criteria") {
            let ac_section = "\n\n## Acceptance Criteria\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
            content.push_str(ac_section);
            return Some(format!(
                "{}: Added Acceptance Criteria with scenario",
                error.file.display()
            ));
        }

        // Find Acceptance Criteria section
        let ac_marker = "## Acceptance Criteria";
        if let Some(ac_pos) = content.find(ac_marker) {
            // Check if there are any scenarios with WHEN/THEN
            let after_ac = &content[ac_pos..];
            let has_scenario = after_ac.contains("### Scenario:") || after_ac.contains("#### Scenario:");
            let has_when_then = after_ac.contains("WHEN") && after_ac.contains("THEN");

            if !has_scenario || !has_when_then {
                // Add a placeholder scenario
                let ac_end = ac_pos + ac_marker.len();
                let scenario = "\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
                content.insert_str(ac_end, scenario);
                return Some(format!(
                    "{}: Added placeholder scenario to Acceptance Criteria",
                    error.file.display()
                ));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_fix_missing_overview() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        std::fs::write(&file_path, "# Spec: Test\n\n## Requirements\n\nSome content").unwrap();

        let fixer = AutoFixer::new(temp_dir.path());
        let error = ValidationError::new(
            "Missing required heading: Overview",
            file_path.clone(),
            None,
            crate::models::Severity::High,
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[error]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        assert_eq!(result.files_modified, 1);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Overview"));
    }

    #[test]
    fn test_fix_missing_acceptance_criteria() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        std::fs::write(&file_path, "# Spec: Test\n\n## Overview\n\nSome content").unwrap();

        let fixer = AutoFixer::new(temp_dir.path());
        let error = ValidationError::new(
            "Missing required heading: Acceptance Criteria",
            file_path.clone(),
            None,
            crate::models::Severity::High,
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[error]).unwrap();
        assert_eq!(result.errors_fixed, 1);

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("WHEN"));
        assert!(content.contains("THEN"));
    }
}
