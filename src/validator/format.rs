use crate::models::{ErrorCategory, Severity, ValidationError, ValidationResult, ValidationRules};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::path::{Path, PathBuf};

/// AST-based format validator for spec files
pub struct SpecFormatValidator {
    rules: ValidationRules,
}

impl SpecFormatValidator {
    /// Create a new format validator with rules
    pub fn new(rules: ValidationRules) -> Self {
        Self { rules }
    }

    /// Validate a spec file's format
    pub fn validate(&self, file_path: &Path) -> ValidationResult {
        let mut errors = Vec::new();

        // Read file content
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Failed to read file: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Check for empty content
        if content.trim().is_empty() {
            errors.push(ValidationError::new(
                "File is empty",
                file_path,
                None,
                Severity::High,
                ErrorCategory::EmptyContent,
            ));
            return ValidationResult::new(errors);
        }

        // Parse markdown with offset tracking
        let parser = Parser::new_ext(&content, Options::all());
        let mut state = ValidationState::new(file_path.to_path_buf());

        // Track line numbers manually (pulldown-cmark doesn't provide this directly)
        let lines: Vec<&str> = content.lines().collect();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let level_num = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    state.enter_heading(level_num);
                }
                Event::End(TagEnd::Heading(_)) => {
                    state.exit_heading();
                }
                Event::Text(text) => {
                    if let Some(heading_level) = state.current_heading_level {
                        let text_str = text.as_ref();

                        // Track headings
                        state.add_heading(heading_level, text_str.to_string());

                        // Check for requirement pattern (### R\d+:)
                        if heading_level == 3 {
                            let line_num = find_line_number(&lines, text_str);
                            errors.extend(self.validate_requirement_heading(
                                text_str,
                                line_num,
                                &state,
                            ));
                        }

                        // Check for scenario pattern (#### Scenario:)
                        if heading_level == 4 {
                            let line_num = find_line_number(&lines, text_str);
                            errors.extend(self.validate_scenario_heading(
                                text_str,
                                line_num,
                                &state,
                            ));
                        }
                    }

                    // Check for WHEN/THEN patterns in content
                    if self.rules.require_when_then && state.in_requirement {
                        let text_str = text.as_ref();
                        if text_str.contains("**WHEN**") {
                            state.has_when = true;
                        }
                        if text_str.contains("**THEN**") {
                            state.has_then = true;
                        }
                    }
                }
                Event::Start(Tag::List(..)) => {
                    state.in_requirement = true;
                }
                _ => {}
            }
        }

        // Validate required headings
        errors.extend(self.validate_required_headings(&state));

        // Validate requirement completeness
        errors.extend(self.validate_requirement_completeness(&state));

        ValidationResult::new(errors)
    }

    /// Validate requirement heading format
    fn validate_requirement_heading(
        &self,
        text: &str,
        line_num: Option<usize>,
        state: &ValidationState,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        let req_regex = match Regex::new(&self.rules.requirement_pattern) {
            Ok(r) => r,
            Err(_) => return errors, // Invalid regex in config
        };

        let full_heading = format!("### {}", text);
        if !req_regex.is_match(&full_heading) {
            let severity = self.rules.severity_map.invalid_requirement_format;
            errors.push(ValidationError::new(
                format!(
                    "Requirement heading '{}' doesn't match pattern '{}'",
                    text, self.rules.requirement_pattern
                ),
                state.file_path.clone(),
                line_num,
                severity,
                ErrorCategory::InvalidRequirementFormat,
            ));
        }

        errors
    }

    /// Validate scenario heading format
    fn validate_scenario_heading(
        &self,
        text: &str,
        line_num: Option<usize>,
        state: &ValidationState,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        let scenario_regex = match Regex::new(&self.rules.scenario_pattern) {
            Ok(r) => r,
            Err(_) => return errors,
        };

        let full_heading = format!("#### {}", text);
        if !scenario_regex.is_match(&full_heading) {
            let severity = self.rules.severity_map.missing_scenario;
            errors.push(ValidationError::new(
                format!(
                    "Scenario heading '{}' doesn't match pattern '{}'",
                    text, self.rules.scenario_pattern
                ),
                state.file_path.clone(),
                line_num,
                severity,
                ErrorCategory::MissingScenario,
            ));
        }

        errors
    }

    /// Validate required headings are present
    fn validate_required_headings(&self, state: &ValidationState) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for required_heading in &self.rules.required_headings {
            if !state.headings.iter().any(|(_, text)| {
                // Normalize heading comparison (trim, case-insensitive)
                let normalized_found = text.trim().to_lowercase();
                let normalized_required = required_heading.trim().to_lowercase();
                normalized_found == normalized_required
                    || normalized_found.starts_with(&normalized_required)
            }) {
                let severity = self.rules.severity_map.missing_heading;
                errors.push(ValidationError::new(
                    format!("Missing required heading: {}", required_heading),
                    state.file_path.clone(),
                    None,
                    severity,
                    ErrorCategory::MissingHeading,
                ));
            }
        }

        errors
    }

    /// Validate requirement completeness (scenarios, WHEN/THEN)
    fn validate_requirement_completeness(&self, state: &ValidationState) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Count scenarios (level 4 headings)
        let scenario_count = state.headings.iter().filter(|(level, _)| *level == 4).count();

        if scenario_count < self.rules.scenario_min_count {
            let severity = self.rules.severity_map.missing_scenario;
            errors.push(ValidationError::new(
                format!(
                    "Found {} scenarios, but minimum {} required",
                    scenario_count, self.rules.scenario_min_count
                ),
                state.file_path.clone(),
                None,
                severity,
                ErrorCategory::MissingScenario,
            ));
        }

        // Validate WHEN/THEN presence
        if self.rules.require_when_then && state.in_requirement {
            if !state.has_when {
                let severity = self.rules.severity_map.missing_when_then;
                errors.push(ValidationError::new(
                    "Missing **WHEN** clause in scenarios",
                    state.file_path.clone(),
                    None,
                    severity,
                    ErrorCategory::MissingWhenThen,
                ));
            }
            if !state.has_then {
                let severity = self.rules.severity_map.missing_when_then;
                errors.push(ValidationError::new(
                    "Missing **THEN** clause in scenarios",
                    state.file_path.clone(),
                    None,
                    severity,
                    ErrorCategory::MissingWhenThen,
                ));
            }
        }

        errors
    }
}

/// Validation state tracking during parsing
struct ValidationState {
    file_path: PathBuf,
    current_heading_level: Option<usize>,
    headings: Vec<(usize, String)>,
    in_requirement: bool,
    has_when: bool,
    has_then: bool,
}

impl ValidationState {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            current_heading_level: None,
            headings: Vec::new(),
            in_requirement: false,
            has_when: false,
            has_then: false,
        }
    }

    fn enter_heading(&mut self, level: usize) {
        self.current_heading_level = Some(level);
    }

    fn exit_heading(&mut self) {
        self.current_heading_level = None;
    }

    fn add_heading(&mut self, level: usize, text: String) {
        self.headings.push((level, text));
    }
}

/// Find line number for a given text in the file
fn find_line_number(lines: &[&str], text: &str) -> Option<usize> {
    for (idx, line) in lines.iter().enumerate() {
        if line.contains(text) {
            return Some(idx + 1); // 1-indexed
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_spec() {
        let content = r#"# Specification: Test Feature

## Overview
This is a test specification.

## Requirements

### R1: First Requirement
This is the first requirement.

#### Scenario: Success Case
- **WHEN** user performs action
- **THEN** system responds correctly
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SpecFormatValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(result.is_valid(), "Expected valid spec, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_missing_required_heading() {
        let content = r#"# Specification: Test Feature

## Requirements
### R1: First Requirement
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SpecFormatValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e.category,
            ErrorCategory::MissingHeading
        )));
    }

    #[test]
    fn test_invalid_requirement_format() {
        let content = r#"# Specification: Test Feature

## Overview
Test

## Requirements

### Bad Format
This doesn't follow R\d+: pattern
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SpecFormatValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e.category,
            ErrorCategory::InvalidRequirementFormat
        )));
    }

    #[test]
    fn test_empty_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"").unwrap();
        file.flush().unwrap();

        let validator = SpecFormatValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(
            e.category,
            ErrorCategory::EmptyContent
        )));
    }
}
