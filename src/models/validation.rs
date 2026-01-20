use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Document type for type-specific validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// PRD (Product Requirements Document) - proposal.md
    Prd,
    /// Task list - tasks.md
    Task,
    /// Technical Design Spec - specs/*.md
    Spec,
}

impl DocumentType {
    /// Determine document type from file path
    pub fn from_path(path: &std::path::Path) -> Self {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name == "proposal.md" {
            DocumentType::Prd
        } else if file_name == "tasks.md" {
            DocumentType::Task
        } else {
            // Default to Spec for files in specs/ directory or any other .md files
            DocumentType::Spec
        }
    }
}

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
    /// Cross-file inconsistency (spec_ref mismatch, orphan specs)
    Inconsistency,
    /// Circular dependency in task graph
    CircularDependency,
}

impl ErrorCategory {
    /// Check if this error category can be automatically fixed
    pub fn is_fixable(&self) -> bool {
        matches!(
            self,
            ErrorCategory::MissingHeading
                | ErrorCategory::MissingWhenThen
                | ErrorCategory::MissingScenario
        )
    }

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
            ErrorCategory::Inconsistency => "Inconsistency",
            ErrorCategory::CircularDependency => "Circular Dependency",
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
        // Default rules are for Spec documents (most restrictive)
        Self::for_spec()
    }
}

impl ValidationRules {
    /// Get validation rules for PRD (proposal.md)
    /// PRD documents are less strict - they describe "why" and "what", not technical details
    pub fn for_prd() -> Self {
        Self {
            required_headings: vec![
                // PRD doesn't require spec-style headings
                // Common PRD sections are: Summary, Why, What Changes, Impact
            ],
            requirement_pattern: String::new(), // No requirement pattern for PRD
            scenario_pattern: String::new(),    // No scenarios required
            scenario_min_count: 0,              // No minimum scenarios
            require_when_then: false,           // No WHEN/THEN required
            when_pattern: String::new(),
            then_pattern: String::new(),
            severity_map: SeverityMap::default(),
        }
    }

    /// Get validation rules for Task list (tasks.md)
    /// Task documents describe implementation tasks, not formal specifications
    pub fn for_task() -> Self {
        Self {
            required_headings: vec![
                // Task files should have a "Tasks" heading or numbered sections
                // But we'll be lenient and not require specific headings
            ],
            requirement_pattern: String::new(), // No formal requirements
            scenario_pattern: String::new(),    // No scenarios in task files
            scenario_min_count: 0,              // No minimum scenarios
            require_when_then: false,           // No WHEN/THEN required
            when_pattern: String::new(),
            then_pattern: String::new(),
            severity_map: SeverityMap::default(),
        }
    }

    /// Get validation rules for Spec (Technical Design) documents
    ///
    /// Spec format (TD + AC):
    /// - # Spec: [Feature Name]
    /// - ## Overview
    /// - ## Flow (Mermaid diagrams)
    /// - ## Data Model (JSON Schema)
    /// - ## Interfaces (pseudo code)
    /// - ## Acceptance Criteria (WHEN/THEN)
    pub fn for_spec() -> Self {
        // Use central spec format rules
        let spec_rules = crate::models::spec_rules::SpecFormatRules::spec_defaults();

        Self {
            required_headings: spec_rules.required_headings.clone(),
            requirement_pattern: spec_rules.requirement_pattern.clone().unwrap_or_default(),
            // Use regex pattern from central rules (supports multiline format)
            scenario_pattern: spec_rules.scenario_regex_pattern(),
            scenario_min_count: spec_rules.min_scenarios,
            require_when_then: spec_rules.require_when_then,
            // Flexible patterns - match both plain "WHEN" and bold "**WHEN**"
            // Use \* to match literal asterisk (in raw string, \* becomes literal \* in regex)
            when_pattern: format!(r"\*\*{}\*\*|{}", spec_rules.when_keyword, spec_rules.when_keyword),
            then_pattern: format!(r"\*\*{}\*\*|{}", spec_rules.then_keyword, spec_rules.then_keyword),
            severity_map: SeverityMap::default(),
        }
    }

    /// Get validation rules based on document type
    pub fn for_document_type(doc_type: DocumentType) -> Self {
        match doc_type {
            DocumentType::Prd => Self::for_prd(),
            DocumentType::Task => Self::for_task(),
            DocumentType::Spec => Self::for_spec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_rules_use_central_format() {
        let rules = ValidationRules::for_spec();

        // Should use the multiline pattern
        assert!(rules.scenario_pattern.contains("(?m)"),
            "Spec rules should use multiline pattern, got: {}", rules.scenario_pattern);
        assert!(rules.scenario_pattern.contains("###"),
            "Spec rules should check for ### Scenario headings");

        // Should require at least 1 scenario
        assert_eq!(rules.scenario_min_count, 1, "Spec should require at least 1 scenario");

        // Should require WHEN/THEN
        assert!(rules.require_when_then, "Spec should require WHEN/THEN");
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
            ErrorCategory::Inconsistency => Severity::High,
            ErrorCategory::CircularDependency => Severity::High,
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

/// Validation CLI options
#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// Treat warnings (MEDIUM/LOW) as errors
    pub strict: bool,
    /// Show verbose output with additional details
    pub verbose: bool,
    /// Output results as JSON
    pub json: bool,
    /// Attempt to auto-fix fixable errors
    pub fix: bool,
}

impl ValidationOptions {
    /// Create new options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set JSON output mode
    pub fn with_json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    /// Set fix mode
    pub fn with_fix(mut self, fix: bool) -> Self {
        self.fix = fix;
        self
    }
}

/// JSON output format for validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationJsonOutput {
    /// Whether validation passed
    pub valid: bool,
    /// Counts by severity
    pub counts: ValidationCounts,
    /// List of all errors
    pub errors: Vec<JsonValidationError>,
    /// Stale files detected
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stale_files: Vec<String>,
}

/// Validation counts by severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCounts {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// JSON-friendly validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonValidationError {
    pub severity: String,
    pub category: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl From<&ValidationError> for JsonValidationError {
    fn from(error: &ValidationError) -> Self {
        Self {
            severity: error.severity.name().to_string(),
            category: error.category.name().to_string(),
            message: error.message.clone(),
            file: Some(error.file.display().to_string()),
            line: error.line,
        }
    }
}
