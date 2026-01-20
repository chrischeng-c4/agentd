//! Central Spec Format Rules
//!
//! This module defines the canonical format rules for agentd specifications.
//! Both MCP tools and validators derive their rules from this single source of truth.

use serde::{Deserialize, Serialize};

/// Format style for scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioFormat {
    /// WHEN and THEN on same line (e.g., "WHEN x THEN y")
    SingleLine,
    /// WHEN and THEN on separate lines (bullet points)
    MultiLine,
}

/// Central specification format rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFormatRules {
    /// Document type these rules apply to
    pub document_type: DocumentType,

    /// Required top-level headings
    pub required_headings: Vec<String>,

    /// Scenario format style
    pub scenario_format: ScenarioFormat,

    /// Heading level for scenarios (2 = ##, 3 = ###, etc.)
    pub scenario_heading_level: u8,

    /// Scenario heading prefix (e.g., "Scenario:")
    pub scenario_heading_prefix: String,

    /// Minimum number of scenarios required
    pub min_scenarios: usize,

    /// WHEN keyword (for localization/customization)
    pub when_keyword: String,

    /// THEN keyword (for localization/customization)
    pub then_keyword: String,

    /// Whether scenarios must have both WHEN and THEN
    pub require_when_then: bool,

    /// Requirement heading pattern (e.g., "R1:", "R2:")
    pub requirement_pattern: Option<String>,
}

/// Document type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// Product Requirements Document (proposal.md)
    Prd,
    /// Technical Specification (specs/*.md)
    Spec,
    /// Task breakdown (tasks.md)
    Task,
}

impl SpecFormatRules {
    /// Get default rules for PRD documents
    pub fn prd_defaults() -> Self {
        Self {
            document_type: DocumentType::Prd,
            required_headings: vec![
                "Summary".to_string(),
                "Why".to_string(),
                "What Changes".to_string(),
                "Impact".to_string(),
            ],
            scenario_format: ScenarioFormat::MultiLine,
            scenario_heading_level: 3,
            scenario_heading_prefix: "Scenario:".to_string(),
            min_scenarios: 0, // PRD doesn't require scenarios
            when_keyword: "WHEN".to_string(),
            then_keyword: "THEN".to_string(),
            require_when_then: false,
            requirement_pattern: None,
        }
    }

    /// Get default rules for Spec documents
    pub fn spec_defaults() -> Self {
        Self {
            document_type: DocumentType::Spec,
            required_headings: vec![
                "Overview".to_string(),
                "Acceptance Criteria".to_string(),
            ],
            scenario_format: ScenarioFormat::MultiLine,
            scenario_heading_level: 3,
            scenario_heading_prefix: "Scenario:".to_string(),
            min_scenarios: 1, // At least one scenario required
            when_keyword: "WHEN".to_string(),
            then_keyword: "THEN".to_string(),
            require_when_then: true,
            requirement_pattern: None, // Allow flexible requirement headings
        }
    }

    /// Get default rules for Task documents
    pub fn task_defaults() -> Self {
        Self {
            document_type: DocumentType::Task,
            required_headings: vec![], // Flexible task structure
            scenario_format: ScenarioFormat::MultiLine,
            scenario_heading_level: 3,
            scenario_heading_prefix: "Scenario:".to_string(),
            min_scenarios: 0,
            when_keyword: "WHEN".to_string(),
            then_keyword: "THEN".to_string(),
            require_when_then: false,
            requirement_pattern: None,
        }
    }

    /// Get rules for a specific document type
    pub fn for_document_type(doc_type: DocumentType) -> Self {
        match doc_type {
            DocumentType::Prd => Self::prd_defaults(),
            DocumentType::Spec => Self::spec_defaults(),
            DocumentType::Task => Self::task_defaults(),
        }
    }

    /// Generate regex pattern for matching scenarios based on format
    pub fn scenario_regex_pattern(&self) -> String {
        let heading_hashes = "#".repeat(self.scenario_heading_level as usize);
        let prefix = &self.scenario_heading_prefix;

        match self.scenario_format {
            ScenarioFormat::SingleLine => {
                // Old format: WHEN...THEN on same line
                format!(r"{}\s*{}\s+.*?{}.*?{}",
                    heading_hashes, prefix, self.when_keyword, self.then_keyword)
            }
            ScenarioFormat::MultiLine => {
                // New format: Support both explicit scenario headings AND inline WHEN/THEN bullets
                // Match either:
                // 1. ### Scenario: heading (new format - simple match, just check heading exists)
                // 2. - WHEN...THEN pattern (old/compact format - single line bullet)
                // Use (?m) for multiline mode to match ^ and $ at line boundaries
                format!(r"(?m)^{}\s*{}|^-\s*{}[^\n]*{}",
                    heading_hashes, prefix, self.when_keyword, self.then_keyword)
            }
        }
    }

    /// Generate skeleton markdown template for this document type
    pub fn to_markdown_skeleton(&self) -> String {
        match self.document_type {
            DocumentType::Spec => self.spec_markdown_skeleton(),
            DocumentType::Prd => self.prd_markdown_skeleton(),
            DocumentType::Task => self.task_markdown_skeleton(),
        }
    }

    fn spec_markdown_skeleton(&self) -> String {
        let heading_hashes = "#".repeat(self.scenario_heading_level as usize);

        format!(
            r#"# Specification: [Feature Name]

## Overview
[Brief description of what this spec covers and why it exists]

## Requirements

### R1: [Requirement Title]
[Description of what this requirement must do]

### R2: [Another Requirement]
[Description]

## Acceptance Criteria

{} {prefix} [Scenario Name]
- **{when}** [condition or action]
- **{then}** [expected outcome]

{} {prefix} [Another Scenario]
- **{when}** [condition]
- **{then}** [result]

{} {prefix} [Edge Case Scenario]
- **{when}** [edge condition]
- **{then}** [expected behavior]
"#,
            heading_hashes,
            heading_hashes,
            heading_hashes,
            prefix = self.scenario_heading_prefix,
            when = self.when_keyword,
            then = self.then_keyword,
        )
    }

    fn prd_markdown_skeleton(&self) -> String {
        r#"# Change: [change-id]

## Summary
[1-2 sentence description of the change]

## Why
[Problem statement and business motivation]

## What Changes
- [List of concrete changes]
- [Group by area: API, UI, Database, etc.]

## Impact
- Affected specs: [list]
- Affected code: [file paths]
- Breaking changes: [Yes/No with explanation]
"#.to_string()
    }

    fn task_markdown_skeleton(&self) -> String {
        r#"# Tasks

## Layer: [layer name]

- [ ] [Task ID] [Task name]
  - File: `path/to/file` (CREATE/MODIFY/DELETE)
  - Spec: `spec-name#section`
  - Do: [Description of what to implement]
  - Depends: [dependencies]
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_rules_defaults() {
        let rules = SpecFormatRules::spec_defaults();

        assert_eq!(rules.document_type, DocumentType::Spec);
        assert_eq!(rules.scenario_format, ScenarioFormat::MultiLine);
        assert_eq!(rules.min_scenarios, 1);
        assert!(rules.require_when_then);
        assert_eq!(rules.when_keyword, "WHEN");
        assert_eq!(rules.then_keyword, "THEN");
    }

    #[test]
    fn test_scenario_regex_multiline() {
        use regex::Regex;

        let rules = SpecFormatRules::spec_defaults();
        let pattern = rules.scenario_regex_pattern();

        // Should match scenario heading
        assert!(pattern.contains("###"));
        assert!(pattern.contains("Scenario:"));

        // Test with actual content
        let content = r#"### Scenario: Add two positive integers
- **WHEN** calling `add(10, 5)`
- **THEN** the result should be `15`

### Scenario: Add negative numbers
- **WHEN** calling `add(-5, -3)`
- **THEN** the result should be `-8`"#;

        let regex = Regex::new(&pattern).expect("Invalid regex pattern");
        let matches: Vec<_> = regex.find_iter(content).collect();

        assert!(matches.len() >= 2, "Should find at least 2 scenarios, found {}: {:?}", matches.len(), matches);
    }

    #[test]
    fn test_markdown_skeleton_generation() {
        let rules = SpecFormatRules::spec_defaults();
        let skeleton = rules.to_markdown_skeleton();

        assert!(skeleton.contains("## Overview"));
        assert!(skeleton.contains("## Acceptance Criteria"));
        assert!(skeleton.contains("### Scenario:"));
        assert!(skeleton.contains("**WHEN**"));
        assert!(skeleton.contains("**THEN**"));
    }

    #[test]
    fn test_document_type_specific_rules() {
        let prd = SpecFormatRules::for_document_type(DocumentType::Prd);
        let spec = SpecFormatRules::for_document_type(DocumentType::Spec);
        let task = SpecFormatRules::for_document_type(DocumentType::Task);

        assert_eq!(prd.min_scenarios, 0);
        assert_eq!(spec.min_scenarios, 1);
        assert_eq!(task.min_scenarios, 0);

        assert!(!prd.require_when_then);
        assert!(spec.require_when_then);
        assert!(!task.require_when_then);
    }
}
