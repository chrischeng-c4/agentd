//! JSON Schema Validation for Frontmatter
//!
//! Validates YAML frontmatter against JSON Schema definitions.

use crate::models::{ErrorCategory, Severity, ValidationError, ValidationResult};
use crate::parser::frontmatter::{normalize_content, split_frontmatter};
use anyhow::{Context, Result};
use jsonschema::Validator;
use serde_json::Value as JsonValue;
use std::path::Path;

/// Document types for schema validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentType {
    Proposal,
    Tasks,
    Spec,
    Challenge,
    State,
}

impl DocumentType {
    /// Get the schema filename for this document type
    pub fn schema_filename(&self) -> &'static str {
        match self {
            DocumentType::Proposal => "proposal.schema.json",
            DocumentType::Tasks => "tasks.schema.json",
            DocumentType::Spec => "spec.schema.json",
            DocumentType::Challenge => "challenge.schema.json",
            DocumentType::State => "state.schema.json",
        }
    }

    /// Detect document type from filename
    pub fn from_filename(filename: &str) -> Option<Self> {
        let lower = filename.to_lowercase();
        if lower == "proposal.md" {
            Some(DocumentType::Proposal)
        } else if lower == "tasks.md" {
            Some(DocumentType::Tasks)
        } else if lower == "challenge.md" {
            Some(DocumentType::Challenge)
        } else if lower == "state.yaml" || lower == "state.yml" {
            Some(DocumentType::State)
        } else if lower.ends_with(".md") {
            // Default to spec for other .md files in specs/ directory
            Some(DocumentType::Spec)
        } else {
            None
        }
    }

    /// Detect document type from frontmatter type field
    pub fn from_type_field(type_field: &str) -> Option<Self> {
        match type_field.to_lowercase().as_str() {
            "proposal" => Some(DocumentType::Proposal),
            "tasks" => Some(DocumentType::Tasks),
            "spec" => Some(DocumentType::Spec),
            "challenge" => Some(DocumentType::Challenge),
            "state" => Some(DocumentType::State),
            _ => None,
        }
    }
}

/// Schema validator for frontmatter
pub struct SchemaValidator {
    schemas_dir: std::path::PathBuf,
    /// Cached compiled validators
    validators: std::collections::HashMap<DocumentType, Validator>,
}

impl SchemaValidator {
    /// Create a new schema validator
    ///
    /// # Arguments
    /// * `schemas_dir` - Directory containing JSON Schema files
    pub fn new(schemas_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            schemas_dir: schemas_dir.into(),
            validators: std::collections::HashMap::new(),
        }
    }

    /// Load and compile a schema for a document type
    fn get_validator(&mut self, doc_type: DocumentType) -> Result<&Validator> {
        if !self.validators.contains_key(&doc_type) {
            let schema_path = self.schemas_dir.join(doc_type.schema_filename());
            let schema_content = std::fs::read_to_string(&schema_path)
                .with_context(|| format!("Failed to read schema: {}", schema_path.display()))?;

            let schema: JsonValue = serde_json::from_str(&schema_content)
                .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

            let validator = Validator::new(&schema)
                .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;

            self.validators.insert(doc_type, validator);
        }

        Ok(self.validators.get(&doc_type).unwrap())
    }

    /// Validate a file's frontmatter against its schema
    pub fn validate_file(&mut self, file_path: &Path) -> ValidationResult {
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

        // Validate content
        errors.extend(self.validate_content(&content, file_path).errors);
        ValidationResult::new(errors)
    }

    /// Validate content string (for testing or direct validation)
    pub fn validate_content(&mut self, content: &str, file_path: &Path) -> ValidationResult {
        let mut errors = Vec::new();

        // Normalize and extract frontmatter
        let normalized = normalize_content(content);
        let (frontmatter_str, _) = match split_frontmatter(&normalized) {
            Ok((fm, body)) => (fm, body),
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Invalid frontmatter format: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Parse YAML to JSON for schema validation
        let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&frontmatter_str) {
            Ok(v) => v,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Invalid YAML in frontmatter: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Convert YAML to JSON for jsonschema validation
        let json_value: JsonValue = match serde_json::to_value(&yaml_value) {
            Ok(v) => v,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Failed to convert YAML to JSON: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Determine document type
        let doc_type = self.detect_document_type(&json_value, file_path);
        let doc_type = match doc_type {
            Some(dt) => dt,
            None => {
                errors.push(ValidationError::new(
                    "Cannot determine document type from frontmatter or filename",
                    file_path,
                    None,
                    Severity::Medium,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Get validator and validate
        let validator = match self.get_validator(doc_type) {
            Ok(v) => v,
            Err(e) => {
                errors.push(ValidationError::new(
                    format!("Failed to load schema: {}", e),
                    file_path,
                    None,
                    Severity::High,
                    ErrorCategory::InvalidStructure,
                ));
                return ValidationResult::new(errors);
            }
        };

        // Run validation using iter_errors to get all validation errors
        for error in validator.iter_errors(&json_value) {
            let path = error.instance_path.to_string();
            let message = if path.is_empty() {
                error.to_string()
            } else {
                format!("{}: {}", path, error)
            };

            // Determine severity based on error type
            let severity = if message.contains("required") {
                Severity::High
            } else if message.contains("type") || message.contains("enum") {
                Severity::High
            } else {
                Severity::Medium
            };

            errors.push(ValidationError::new(
                message,
                file_path,
                None,
                severity,
                ErrorCategory::InvalidStructure,
            ));
        }

        ValidationResult::new(errors)
    }

    /// Detect document type from frontmatter or filename
    fn detect_document_type(&self, json_value: &JsonValue, file_path: &Path) -> Option<DocumentType> {
        // First try to detect from frontmatter "type" field
        if let Some(type_field) = json_value.get("type").and_then(|v| v.as_str()) {
            if let Some(doc_type) = DocumentType::from_type_field(type_field) {
                return Some(doc_type);
            }
        }

        // Fall back to filename detection
        if let Some(filename) = file_path.file_name().and_then(|f| f.to_str()) {
            return DocumentType::from_filename(filename);
        }

        None
    }

    /// Validate frontmatter has all required fields for a document type
    pub fn validate_required_fields(
        &self,
        frontmatter: &serde_yaml::Value,
        doc_type: DocumentType,
        file_path: &Path,
    ) -> ValidationResult {
        let mut errors = Vec::new();

        let required_fields: &[&str] = match doc_type {
            DocumentType::Proposal => &["id", "type", "version", "status"],
            DocumentType::Tasks => &["id", "type", "version"],
            DocumentType::Spec => &["id", "type", "title", "version"],
            DocumentType::Challenge => &["id", "type", "version", "verdict"],
            DocumentType::State => &["change_id", "phase"],
        };

        if let serde_yaml::Value::Mapping(map) = frontmatter {
            for field in required_fields {
                let key = serde_yaml::Value::String(field.to_string());
                if !map.contains_key(&key) {
                    errors.push(ValidationError::new(
                        format!("Missing required field: {}", field),
                        file_path,
                        None,
                        Severity::High,
                        ErrorCategory::InvalidStructure,
                    ));
                }
            }
        } else {
            errors.push(ValidationError::new(
                "Frontmatter must be a YAML mapping",
                file_path,
                None,
                Severity::High,
                ErrorCategory::InvalidStructure,
            ));
        }

        ValidationResult::new(errors)
    }
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Quick validation of a file against its schema
///
/// Uses the default schemas directory (agentd/schemas)
pub fn validate_frontmatter_schema(
    file_path: &Path,
    project_root: &Path,
) -> ValidationResult {
    let schemas_dir = project_root.join("agentd/schemas");
    let mut validator = SchemaValidator::new(schemas_dir);
    validator.validate_file(file_path)
}

/// Validate frontmatter content directly (for testing)
pub fn validate_frontmatter_content(
    content: &str,
    doc_type: DocumentType,
    project_root: &Path,
) -> ValidationResult {
    let schemas_dir = project_root.join("agentd/schemas");
    let mut validator = SchemaValidator::new(schemas_dir);

    // Create a temporary path for error reporting
    let temp_path = std::path::PathBuf::from(format!("temp.{}", match doc_type {
        DocumentType::Proposal => "proposal.md",
        DocumentType::Tasks => "tasks.md",
        DocumentType::Spec => "spec.md",
        DocumentType::Challenge => "challenge.md",
        DocumentType::State => "state.yaml",
    }));

    validator.validate_content(content, &temp_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_type_from_filename() {
        assert_eq!(DocumentType::from_filename("proposal.md"), Some(DocumentType::Proposal));
        assert_eq!(DocumentType::from_filename("tasks.md"), Some(DocumentType::Tasks));
        assert_eq!(DocumentType::from_filename("CHALLENGE.md"), Some(DocumentType::Challenge));
        assert_eq!(DocumentType::from_filename("STATE.yaml"), Some(DocumentType::State));
        assert_eq!(DocumentType::from_filename("auth.md"), Some(DocumentType::Spec));
        assert_eq!(DocumentType::from_filename("readme.txt"), None);
    }

    #[test]
    fn test_document_type_from_type_field() {
        assert_eq!(DocumentType::from_type_field("proposal"), Some(DocumentType::Proposal));
        assert_eq!(DocumentType::from_type_field("PROPOSAL"), Some(DocumentType::Proposal));
        assert_eq!(DocumentType::from_type_field("tasks"), Some(DocumentType::Tasks));
        assert_eq!(DocumentType::from_type_field("spec"), Some(DocumentType::Spec));
        assert_eq!(DocumentType::from_type_field("challenge"), Some(DocumentType::Challenge));
        assert_eq!(DocumentType::from_type_field("invalid"), None);
    }

    #[test]
    fn test_schema_filename() {
        assert_eq!(DocumentType::Proposal.schema_filename(), "proposal.schema.json");
        assert_eq!(DocumentType::Tasks.schema_filename(), "tasks.schema.json");
        assert_eq!(DocumentType::Spec.schema_filename(), "spec.schema.json");
        assert_eq!(DocumentType::Challenge.schema_filename(), "challenge.schema.json");
        assert_eq!(DocumentType::State.schema_filename(), "state.schema.json");
    }
}
