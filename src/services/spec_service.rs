//! Spec service - Business logic for spec creation

use crate::models::spec_rules::SpecFormatRules;
use crate::Result;
use chrono::Utc;
use serde_json::Value;
use std::path::Path;

/// Requirement data
#[derive(Debug, Clone)]
pub struct RequirementData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: String,
}

/// Scenario data
#[derive(Debug, Clone)]
pub struct ScenarioData {
    pub name: String,
    pub given: Option<String>,
    pub when: String,
    pub then: String,
}

/// Input structure for creating a spec
#[derive(Debug, Clone)]
pub struct CreateSpecInput {
    pub change_id: String,
    pub spec_id: String,
    pub title: String,
    pub overview: String,
    pub requirements: Vec<RequirementData>,
    pub scenarios: Vec<ScenarioData>,
    pub flow_diagram: Option<String>,
    pub data_model: Option<Value>,
}

/// Create a new spec with validation
pub fn create_spec(input: CreateSpecInput, project_root: &Path) -> Result<String> {
    // Validate spec_id format
    if !input
        .spec_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("spec_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate overview length
    if input.overview.len() < 50 {
        anyhow::bail!("overview must be at least 50 characters");
    }

    // Validate requirements
    if input.requirements.is_empty() {
        anyhow::bail!("At least one requirement is required");
    }

    // Validate scenarios
    if input.scenarios.is_empty() {
        anyhow::bail!("At least one scenario is required");
    }

    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(&input.change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Create proposal first.",
            input.change_id
        );
    }

    // Create specs directory if needed
    let specs_dir = change_dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    // Generate spec content
    let now = Utc::now();
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("id: {}\n", input.spec_id));
    content.push_str("type: spec\n");
    content.push_str(&format!(
        "title: \"{}\"\n",
        input.title.replace('"', "\\\"")
    ));
    content.push_str("version: 1\n");
    content.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));

    // Requirements summary
    let requirement_ids: Vec<String> = input
        .requirements
        .iter()
        .map(|r| r.id.clone())
        .collect();

    content.push_str("requirements:\n");
    content.push_str(&format!("  total: {}\n", input.requirements.len()));
    if !requirement_ids.is_empty() {
        content.push_str("  ids:\n");
        for id in &requirement_ids {
            content.push_str(&format!("    - {}\n", id));
        }
    }

    // Design elements
    content.push_str("design_elements:\n");
    content.push_str(&format!("  has_mermaid: {}\n", input.flow_diagram.is_some()));
    content.push_str(&format!(
        "  has_json_schema: {}\n",
        input.data_model.is_some()
    ));
    content.push_str("  has_pseudo_code: false\n");
    content.push_str("  has_api_spec: false\n");

    content.push_str("---\n\n");

    // Wrap spec content in XML
    content.push_str("<spec>\n\n");

    // Title
    content.push_str(&format!("# {}\n\n", input.title));

    // Overview
    content.push_str("## Overview\n\n");
    content.push_str(&format!("{}\n\n", input.overview));

    // Requirements section
    content.push_str("## Requirements\n\n");

    for req in &input.requirements {
        content.push_str(&format!("### {} - {}\n\n", req.id, req.title));
        content.push_str("```yaml\n");
        content.push_str(&format!("id: {}\n", req.id));
        content.push_str(&format!("priority: {}\n", req.priority));
        content.push_str("status: draft\n");
        content.push_str("```\n\n");
        content.push_str(&format!("{}\n\n", req.description));
    }

    // Acceptance Criteria section - use central format rules
    let spec_rules = SpecFormatRules::spec_defaults();

    // Find the "Acceptance Criteria" heading from required_headings
    let ac_heading = spec_rules
        .required_headings
        .iter()
        .find(|h| h.contains("Acceptance") || h.contains("Criteria"))
        .map(|s| s.as_str())
        .unwrap_or("Acceptance Criteria");

    content.push_str(&format!("## {}\n\n", ac_heading));

    for scenario in &input.scenarios {
        // Use scenario heading format from rules: ### {prefix} {name}
        let heading_hashes = "#".repeat(spec_rules.scenario_heading_level as usize);
        content.push_str(&format!(
            "{} {} {}\n\n",
            heading_hashes, spec_rules.scenario_heading_prefix, scenario.name
        ));

        // Use WHEN/THEN keywords from rules
        if let Some(given_text) = &scenario.given {
            content.push_str(&format!("- **GIVEN** {}\n", given_text));
        }
        content.push_str(&format!(
            "- **{}** {}\n",
            spec_rules.when_keyword, scenario.when
        ));
        content.push_str(&format!(
            "- **{}** {}\n\n",
            spec_rules.then_keyword, scenario.then
        ));
    }

    // Flow diagram (optional)
    if let Some(diagram) = &input.flow_diagram {
        content.push_str("## Flow Diagram\n\n");
        content.push_str("```mermaid\n");
        content.push_str(diagram);
        if !diagram.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("```\n\n");
    }

    // Data model (optional)
    if let Some(model) = &input.data_model {
        content.push_str("## Data Model\n\n");
        content.push_str("```json\n");
        content.push_str(&serde_json::to_string_pretty(model)?);
        content.push_str("\n```\n\n");
    }

    // Close spec XML tag
    content.push_str("</spec>\n");

    // Write the file
    let spec_path = specs_dir.join(format!("{}.md", input.spec_id));
    std::fs::write(&spec_path, &content)?;

    Ok(format!(
        "Created spec '{}' for change '{}' at {}",
        input.spec_id, input.change_id,
        spec_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join("agentd/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let input = CreateSpecInput {
            change_id: "test-change".to_string(),
            spec_id: "mcp-protocol".to_string(),
            title: "MCP Protocol Implementation".to_string(),
            overview: "This specification covers the implementation of the Model Context Protocol (MCP) server for agentd, providing structured tools for proposal generation.".to_string(),
            requirements: vec![
                RequirementData {
                    id: "R1".to_string(),
                    title: "JSON-RPC 2.0 Support".to_string(),
                    description: "The server must support JSON-RPC 2.0 protocol over stdio".to_string(),
                    priority: "high".to_string(),
                },
                RequirementData {
                    id: "R2".to_string(),
                    title: "Tool Registration".to_string(),
                    description: "Tools must be registered and callable via tools/call method".to_string(),
                    priority: "high".to_string(),
                },
            ],
            scenarios: vec![
                ScenarioData {
                    name: "Server Initialization".to_string(),
                    given: Some("MCP client is connected".to_string()),
                    when: "Client sends initialize request".to_string(),
                    then: "Server responds with capabilities".to_string(),
                },
                ScenarioData {
                    name: "Tool Execution".to_string(),
                    given: None,
                    when: "Client calls create_proposal tool".to_string(),
                    then: "Server creates proposal.md and returns success".to_string(),
                },
            ],
            flow_diagram: Some("graph LR\n    A[Client] --> B[Server]\n    B --> C[Tool Registry]\n    C --> D[Execute Tool]".to_string()),
            data_model: None,
        };

        let result = create_spec(input, project_root).unwrap();
        assert!(result.contains("Created spec"));

        // Verify file was created
        let spec_path = project_root.join("agentd/changes/test-change/specs/mcp-protocol.md");
        assert!(spec_path.exists());

        let content = std::fs::read_to_string(&spec_path).unwrap();
        assert!(content.contains("id: mcp-protocol"));
        assert!(content.contains("## Requirements"));
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("### Scenario:"));
        assert!(content.contains("**WHEN**"));
        assert!(content.contains("**THEN**"));
        assert!(content.contains("```mermaid"));
    }

    #[test]
    fn test_create_spec_validation() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Test invalid spec_id
        let input = CreateSpecInput {
            change_id: "test".to_string(),
            spec_id: "Invalid_ID".to_string(),
            title: "Test".to_string(),
            overview: "Test overview that is long enough to pass validation requirements.".to_string(),
            requirements: vec![RequirementData {
                id: "R1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                priority: "medium".to_string(),
            }],
            scenarios: vec![ScenarioData {
                name: "Test".to_string(),
                given: None,
                when: "test".to_string(),
                then: "test".to_string(),
            }],
            flow_diagram: None,
            data_model: None,
        };

        let result = create_spec(input, project_root);
        assert!(result.is_err());
    }
}
