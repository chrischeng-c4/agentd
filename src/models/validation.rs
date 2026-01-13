use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Severity level for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// High severity - blocks archive, must be fixed
    High,
    /// Medium severity - warning, should be fixed
    Medium,
    /// Low severity - informational, nice to fix
    Low,
}

impl Severity {
    /// Get display symbol for severity
    pub fn symbol(&self) -> &'static str {
        match self {
            Severity::High => "🔴",
            Severity::Medium => "🟡",
            Severity::Low => "🔵",
        }
    }

    /// Get display name for severity
    pub fn name(&self) -> &'static str {
        match self {
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

/// A validation error found in a spec file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message describing what's wrong
    pub message: String,
    /// File where the error was found
    pub file: PathBuf,
    /// Line number (1-indexed) if applicable
    pub line: Option<usize>,
    /// Severity level
    pub severity: Severity,
    /// Error category for grouping
    pub category: ErrorCategory,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(
        message: impl Into<String>,
        file: impl Into<PathBuf>,
        line: Option<usize>,
        severity: Severity,
        category: ErrorCategory,
    ) -> Self {
        Self {
            message: message.into(),
            file: file.into(),
            line,
            severity,
            category,
        }
    }

    /// Format error for display
    pub fn format(&self) -> String {
        let file_display = self.file.display();
        if let Some(line) = self.line {
            format!(
                "{} [{}] {}:{} - {}",
                self.severity.symbol(),
                self.severity.name(),
                file_display,
                line,
                self.message
            )
        } else {
            format!(
                "{} [{}] {} - {}",
                self.severity.symbol(),
                self.severity.name(),
                file_display,
                self.message
            )
        }
    }
}

/// Category of validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Missing required heading
    MissingHeading,
    /// Invalid requirement format
    InvalidRequirementFormat,
    /// Missing scenario
    MissingScenario,
    /// Missing WHEN/THEN clause
    MissingWhenThen,
    /// Duplicate requirement ID
    DuplicateRequirement,
    /// Broken cross-reference
    BrokenReference,
    /// Invalid structure
    InvalidStructure,
    /// Empty or incomplete content
    EmptyContent,
}

impl ErrorCategory {
    /// Get display name for category
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::MissingHeading => "Missing Heading",
            ErrorCategory::InvalidRequirementFormat => "Invalid Requirement Format",
            ErrorCategory::MissingScenario => "Missing Scenario",
            ErrorCategory::MissingWhenThen => "Missing WHEN/THEN",
            ErrorCategory::DuplicateRequirement => "Duplicate Requirement",
            ErrorCategory::BrokenReference => "Broken Reference",
            ErrorCategory::InvalidStructure => "Invalid Structure",
            ErrorCategory::EmptyContent => "Empty Content",
        }
    }
}

/// Validation rules loaded from configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Required top-level headings (in order)
    pub required_headings: Vec<String>,
    /// Regex pattern for requirement naming (e.g., "^### R\\d+:")
    pub requirement_pattern: String,
    /// Regex pattern for scenario format
    pub scenario_pattern: String,
    /// Minimum number of scenarios per requirement
    pub scenario_min_count: usize,
    /// Whether to require WHEN/THEN clauses
    pub require_when_then: bool,
    /// Pattern for WHEN clause
    pub when_pattern: String,
    /// Pattern for THEN clause
    pub then_pattern: String,
    /// Severity mapping for different error types
    pub severity_map: SeverityMap,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            required_headings: vec![
                "Specification:".to_string(),
                "Overview".to_string(),
                "Requirements".to_string(),
            ],
            requirement_pattern: r"^R\d+:".to_string(),
            scenario_pattern: r"^Scenario:".to_string(),
            scenario_min_count: 1,
            require_when_then: true,
            when_pattern: r"- \*\*WHEN\*\*".to_string(),
            then_pattern: r"- \*\*THEN\*\*".to_string(),
            severity_map: SeverityMap::default(),
        }
    }
}

/// Mapping of error categories to severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityMap {
    pub missing_heading: Severity,
    pub invalid_requirement_format: Severity,
    pub missing_scenario: Severity,
    pub missing_when_then: Severity,
    pub duplicate_requirement: Severity,
    pub broken_reference: Severity,
}

impl Default for SeverityMap {
    fn default() -> Self {
        Self {
            missing_heading: Severity::High,
            invalid_requirement_format: Severity::High,
            missing_scenario: Severity::High,
            missing_when_then: Severity::High,
            duplicate_requirement: Severity::High,
            broken_reference: Severity::Medium,
        }
    }
}

impl SeverityMap {
    /// Get severity for a given error category
    pub fn get(&self, category: ErrorCategory) -> Severity {
        match category {
            ErrorCategory::MissingHeading => self.missing_heading,
            ErrorCategory::InvalidRequirementFormat => self.invalid_requirement_format,
            ErrorCategory::MissingScenario => self.missing_scenario,
            ErrorCategory::MissingWhenThen => self.missing_when_then,
            ErrorCategory::DuplicateRequirement => self.duplicate_requirement,
            ErrorCategory::BrokenReference => self.broken_reference,
            ErrorCategory::InvalidStructure => Severity::High,
            ErrorCategory::EmptyContent => Severity::High,
        }
    }
}

/// Result of validation
#[derive(Debug)]
pub struct ValidationResult {
    /// List of all validation errors found
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }

    /// Check if validation passed (no high-severity errors)
    pub fn is_valid(&self) -> bool {
        !self.errors.iter().any(|e| e.severity == Severity::High)
    }

    /// Check if there are any errors at all
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Count errors by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.errors.iter().filter(|e| e.severity == severity).count()
    }

    /// Get all high-severity errors (blocking)
    pub fn high_severity_errors(&self) -> Vec<&ValidationError> {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::High)
            .collect()
    }

    /// Format all errors for display
    pub fn format_errors(&self) -> String {
        self.errors
            .iter()
            .map(|e| e.format())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
