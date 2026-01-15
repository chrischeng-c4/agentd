use crate::fillback::strategy::ImportStrategy;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// OpenSpec import strategy
///
/// Converts OpenSpec YAML/JSON files to Agentd Markdown specifications.
/// OpenSpec is a specification format that uses YAML or JSON.
pub struct OpenSpecStrategy;

/// OpenSpec document structure
#[derive(Debug, Deserialize, Serialize)]
struct OpenSpecDocument {
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    requirements: Vec<OpenSpecRequirement>,
    #[serde(default)]
    scenarios: Vec<OpenSpecScenario>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenSpecRequirement {
    id: String,
    description: String,
    #[serde(default)]
    priority: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct OpenSpecScenario {
    name: String,
    #[serde(default)]
    given: Vec<String>,
    #[serde(default)]
    when: String,
    #[serde(default)]
    then: Vec<String>,
}

impl OpenSpecStrategy {
    pub fn new() -> Self {
        Self
    }

    /// Parse OpenSpec from YAML file
    fn parse_yaml(&self, path: &Path) -> Result<OpenSpecDocument> {
        let content = std::fs::read_to_string(path)?;
        let doc: OpenSpecDocument = serde_yaml::from_str(&content)?;
        Ok(doc)
    }

    /// Parse OpenSpec from JSON file
    fn parse_json(&self, path: &Path) -> Result<OpenSpecDocument> {
        let content = std::fs::read_to_string(path)?;
        let doc: OpenSpecDocument = serde_json::from_str(&content)?;
        Ok(doc)
    }

    /// Convert OpenSpec document to Agentd proposal markdown
    fn to_proposal_md(&self, doc: &OpenSpecDocument, change_id: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Change: {}\n\n", change_id));

        output.push_str("## Summary\n");
        if !doc.description.is_empty() {
            output.push_str(&format!("{}\n\n", doc.description));
        } else {
            output.push_str("(Imported from OpenSpec)\n\n");
        }

        output.push_str("## Why\n");
        output.push_str("(To be filled in)\n\n");

        output.push_str("## What Changes\n");
        output.push_str("(To be filled in)\n\n");

        output.push_str("## Impact\n");
        output.push_str("- **Affected Specs:** (To be determined)\n");
        output.push_str("- **Affected Code:** (To be determined)\n");
        output.push_str("- **Breaking Changes:** (To be determined)\n");

        output
    }

    /// Convert OpenSpec document to Agentd spec markdown
    fn to_spec_md(&self, doc: &OpenSpecDocument) -> String {
        let mut output = String::new();

        let name = if !doc.name.is_empty() {
            &doc.name
        } else {
            "Imported Specification"
        };

        output.push_str(&format!("# Specification: {}\n\n", name));

        output.push_str("## Overview\n");
        if !doc.description.is_empty() {
            output.push_str(&format!("{}\n\n", doc.description));
        } else {
            output.push_str("(Imported from OpenSpec)\n\n");
        }

        if !doc.requirements.is_empty() {
            output.push_str("## Requirements\n\n");
            for req in &doc.requirements {
                output.push_str(&format!("### {}\n", req.id));
                output.push_str(&format!("{}\n\n", req.description));
            }
        }

        if !doc.scenarios.is_empty() {
            output.push_str("## Acceptance Criteria\n\n");
            for scenario in &doc.scenarios {
                output.push_str(&format!("### Scenario: {}\n", scenario.name));
                if !scenario.when.is_empty() {
                    output.push_str(&format!("- **WHEN** {}\n", scenario.when));
                }
                for then in &scenario.then {
                    output.push_str(&format!("- **THEN** {}\n", then));
                }
                output.push_str("\n");
            }
        }

        output
    }
}

#[async_trait]
impl ImportStrategy for OpenSpecStrategy {
    async fn execute(&self, source: &Path, change_id: &str) -> Result<()> {
        // Parse the OpenSpec file (try YAML first, then JSON)
        let doc = if source.extension().and_then(|s| s.to_str()) == Some("json") {
            self.parse_json(source)?
        } else {
            self.parse_yaml(source)?
        };

        // Create change directory
        let current_dir = std::env::current_dir()?;
        let change_dir = current_dir.join("agentd/changes").join(change_id);
        std::fs::create_dir_all(&change_dir)?;

        // Generate proposal.md
        let proposal_content = self.to_proposal_md(&doc, change_id);
        std::fs::write(change_dir.join("proposal.md"), proposal_content)?;

        // Generate spec file in specs/ directory
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir)?;

        let spec_content = self.to_spec_md(&doc);
        let spec_filename = if !doc.name.is_empty() {
            format!("{}.md", doc.name.to_lowercase().replace(' ', "_"))
        } else {
            "imported_spec.md".to_string()
        };
        std::fs::write(specs_dir.join(spec_filename), spec_content)?;

        // Generate tasks.md skeleton
        let tasks_content = "# Tasks\n\n## 1. Implementation\n- [ ] 1.1 Implement imported requirements\n  - File: (To be determined)\n  - Do: Review and implement the requirements from the imported spec.\n";
        std::fs::write(change_dir.join("tasks.md"), tasks_content)?;

        Ok(())
    }

    fn can_handle(&self, source: &Path) -> bool {
        if !source.is_file() {
            return false;
        }

        // Check file extension
        if let Some(ext) = source.extension().and_then(|s| s.to_str()) {
            if ext == "yaml" || ext == "yml" || ext == "json" {
                // Try to parse as OpenSpec
                if ext == "json" {
                    return self.parse_json(source).is_ok();
                } else {
                    return self.parse_yaml(source).is_ok();
                }
            }
        }

        false
    }

    fn name(&self) -> &'static str {
        "openspec"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_openspec_yaml_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let openspec_file = temp_dir.path().join("spec.yaml");

        let yaml_content = r#"
name: Test Spec
version: 1.0.0
description: A test specification
requirements:
  - id: R1
    description: First requirement
    priority: high
scenarios:
  - name: Basic Test
    when: user performs action
    then:
      - system responds correctly
"#;
        std::fs::write(&openspec_file, yaml_content).unwrap();

        let strategy = OpenSpecStrategy::new();
        let doc = strategy.parse_yaml(&openspec_file).unwrap();

        assert_eq!(doc.name, "Test Spec");
        assert_eq!(doc.requirements.len(), 1);
        assert_eq!(doc.scenarios.len(), 1);
    }

    #[test]
    fn test_can_handle_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let openspec_file = temp_dir.path().join("spec.yaml");

        let yaml_content = r#"
name: Test
description: Test spec
"#;
        std::fs::write(&openspec_file, yaml_content).unwrap();

        let strategy = OpenSpecStrategy::new();
        assert!(strategy.can_handle(&openspec_file));
    }

    #[test]
    fn test_can_handle_non_openspec() {
        let temp_dir = TempDir::new().unwrap();
        let text_file = temp_dir.path().join("readme.txt");
        std::fs::write(&text_file, "Not an OpenSpec file").unwrap();

        let strategy = OpenSpecStrategy::new();
        assert!(!strategy.can_handle(&text_file));
    }
}
