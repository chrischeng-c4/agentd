use crate::models::{ErrorCategory, Severity, ValidationError, ValidationResult, ValidationRules};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Semantic validator for spec files (duplicates, cross-references, etc.)
pub struct SemanticValidator {
    rules: ValidationRules,
}

impl SemanticValidator {
    /// Create a new semantic validator with rules
    pub fn new(rules: ValidationRules) -> Self {
        Self { rules }
    }

    /// Validate semantic correctness of a spec file
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

        // Extract requirements from the file
        let requirements = self.extract_requirements(&content, file_path);

        // Check for duplicate requirement IDs
        errors.extend(self.check_duplicate_requirements(&requirements, file_path));

        // Check for cross-reference validity
        errors.extend(self.check_cross_references(&content, file_path));

        // Check for empty or incomplete requirements
        errors.extend(self.check_requirement_completeness(&requirements, file_path));

        ValidationResult::new(errors)
    }

    /// Extract requirement IDs from markdown content
    fn extract_requirements(&self, content: &str, file_path: &Path) -> Vec<RequirementInfo> {
        let mut requirements = Vec::new();

        let req_regex = match Regex::new(r"^### (R\d+):(.*)$") {
            Ok(r) => r,
            Err(_) => return requirements,
        };

        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(captures) = req_regex.captures(line) {
                let req_id = captures.get(1).map(|m| m.as_str().to_string());
                let req_title = captures.get(2).map(|m| m.as_str().trim().to_string());

                if let (Some(id), Some(title)) = (req_id, req_title) {
                    requirements.push(RequirementInfo {
                        id,
                        title,
                        line_number: line_num + 1, // 1-indexed
                        file_path: file_path.to_path_buf(),
                    });
                }
            }
        }

        requirements
    }

    /// Check for duplicate requirement IDs
    fn check_duplicate_requirements(
        &self,
        requirements: &[RequirementInfo],
        file_path: &Path,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut seen: HashMap<String, &RequirementInfo> = HashMap::new();

        for req in requirements {
            if let Some(first_occurrence) = seen.get(&req.id) {
                let severity = self.rules.severity_map.duplicate_requirement;
                errors.push(ValidationError::new(
                    format!(
                        "Duplicate requirement ID '{}' (first seen at line {})",
                        req.id, first_occurrence.line_number
                    ),
                    file_path,
                    Some(req.line_number),
                    severity,
                    ErrorCategory::DuplicateRequirement,
                ));
            } else {
                seen.insert(req.id.clone(), req);
            }
        }

        errors
    }

    /// Check cross-references to other spec files
    fn check_cross_references(&self, content: &str, file_path: &Path) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Find all markdown links in the content
        let link_regex = match Regex::new(r"\[([^\]]+)\]\(([^)]+)\)") {
            Ok(r) => r,
            Err(_) => return errors,
        };

        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            for captures in link_regex.captures_iter(line) {
                if let Some(link_target) = captures.get(2) {
                    let target = link_target.as_str();

                    // Only check local file references (not URLs)
                    if !target.starts_with("http://")
                        && !target.starts_with("https://")
                        && !target.starts_with('#')
                    {
                        // Resolve relative path
                        let target_path = if let Some(parent) = file_path.parent() {
                            parent.join(target)
                        } else {
                            PathBuf::from(target)
                        };

                        // Check if file exists
                        if !target_path.exists() {
                            let severity = self.rules.severity_map.broken_reference;
                            errors.push(ValidationError::new(
                                format!("Broken reference to file: {}", target),
                                file_path,
                                Some(line_num + 1),
                                severity,
                                ErrorCategory::BrokenReference,
                            ));
                        }
                    }
                }
            }
        }

        errors
    }

    /// Check for incomplete or empty requirements
    fn check_requirement_completeness(
        &self,
        requirements: &[RequirementInfo],
        file_path: &Path,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for req in requirements {
            // Check for empty titles
            if req.title.is_empty() {
                errors.push(ValidationError::new(
                    format!("Requirement '{}' has empty title", req.id),
                    file_path,
                    Some(req.line_number),
                    Severity::High,
                    ErrorCategory::EmptyContent,
                ));
            }

            // Check for placeholder titles (common pattern)
            let placeholder_patterns = ["TODO", "TBD", "FIXME", "XXX"];
            for pattern in &placeholder_patterns {
                if req.title.to_uppercase().contains(pattern) {
                    errors.push(ValidationError::new(
                        format!(
                            "Requirement '{}' contains placeholder text: '{}'",
                            req.id, req.title
                        ),
                        file_path,
                        Some(req.line_number),
                        Severity::Medium,
                        ErrorCategory::EmptyContent,
                    ));
                    break;
                }
            }
        }

        errors
    }

    /// Validate across multiple spec files (for batch validation)
    pub fn validate_batch(&self, file_paths: &[PathBuf]) -> ValidationResult {
        let mut all_errors = Vec::new();

        // First pass: validate each file individually
        for file_path in file_paths {
            let result = self.validate(file_path);
            all_errors.extend(result.errors);
        }

        // Second pass: cross-file validation
        let cross_file_errors = self.check_cross_file_duplicates(file_paths);
        all_errors.extend(cross_file_errors);

        ValidationResult::new(all_errors)
    }

    /// Check for duplicate requirement IDs across multiple files
    fn check_cross_file_duplicates(&self, file_paths: &[PathBuf]) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut global_requirements: HashMap<String, RequirementInfo> = HashMap::new();

        // Collect all requirements from all files
        for file_path in file_paths {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let requirements = self.extract_requirements(&content, file_path);

                for req in requirements {
                    if let Some(first_occurrence) = global_requirements.get(&req.id) {
                        let severity = self.rules.severity_map.duplicate_requirement;
                        errors.push(ValidationError::new(
                            format!(
                                "Duplicate requirement ID '{}' across files (first seen in {} at line {})",
                                req.id,
                                first_occurrence.file_path.display(),
                                first_occurrence.line_number
                            ),
                            &req.file_path,
                            Some(req.line_number),
                            severity,
                            ErrorCategory::DuplicateRequirement,
                        ));
                    } else {
                        global_requirements.insert(req.id.clone(), req);
                    }
                }
            }
        }

        errors
    }
}

/// Information about a requirement found in a spec file
#[derive(Debug, Clone)]
struct RequirementInfo {
    id: String,
    title: String,
    line_number: usize,
    file_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_no_duplicate_requirements() {
        let content = r#"# Specification: Test Feature

## Requirements

### R1: First Requirement
This is the first requirement.

### R2: Second Requirement
This is the second requirement.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SemanticValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(result.is_valid());
    }

    #[test]
    fn test_duplicate_requirements() {
        let content = r#"# Specification: Test Feature

## Requirements

### R1: First Requirement
This is the first requirement.

### R1: Duplicate Requirement
This is a duplicate.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SemanticValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e.category, ErrorCategory::DuplicateRequirement)));
    }

    #[test]
    fn test_empty_requirement_title() {
        let content = r#"# Specification: Test Feature

## Requirements

### R1:
Empty title requirement.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SemanticValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e.category, ErrorCategory::EmptyContent)));
    }

    #[test]
    fn test_placeholder_in_title() {
        let content = r#"# Specification: Test Feature

## Requirements

### R1: TODO: Implement this feature
Placeholder in title.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SemanticValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        // Should have medium severity error
        assert!(result
            .errors
            .iter()
            .any(|e| e.severity == Severity::Medium
                && matches!(e.category, ErrorCategory::EmptyContent)));
    }

    #[test]
    fn test_broken_reference() {
        let content = r#"# Specification: Test Feature

## Requirements

### R1: First Requirement
See [non-existent file](./non-existent.md) for details.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let validator = SemanticValidator::new(ValidationRules::default());
        let result = validator.validate(file.path());

        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e.category, ErrorCategory::BrokenReference)));
    }
}
