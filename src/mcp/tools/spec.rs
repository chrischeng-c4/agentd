//! create_spec MCP Tool
//!
//! Creates a validated spec file with requirements and acceptance criteria.

use super::{get_optional_string, get_required_array, get_required_string, ToolDefinition};
use crate::models::spec_rules::SpecFormatRules;
use crate::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for create_spec
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_spec".to_string(),
        description: "Create a validated spec file with requirements and acceptance criteria"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id", "spec_id", "title", "overview", "requirements", "scenarios"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID this spec belongs to"
                },
                "spec_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Unique identifier for this spec (lowercase, hyphens allowed)"
                },
                "title": {
                    "type": "string",
                    "description": "Human-readable title for the spec"
                },
                "overview": {
                    "type": "string",
                    "minLength": 50,
                    "description": "Overview of what this spec covers"
                },
                "requirements": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "description"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^R\\d+$",
                                "description": "Requirement ID (e.g., R1, R2)"
                            },
                            "title": {
                                "type": "string",
                                "description": "Short requirement title"
                            },
                            "description": {
                                "type": "string",
                                "description": "Detailed requirement description"
                            },
                            "priority": {
                                "enum": ["high", "medium", "low"],
                                "default": "medium",
                                "description": "Requirement priority"
                            }
                        }
                    },
                    "description": "List of requirements"
                },
                "scenarios": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["name", "when", "then"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Scenario name"
                            },
                            "given": {
                                "type": "string",
                                "description": "Optional precondition"
                            },
                            "when": {
                                "type": "string",
                                "description": "Action or trigger"
                            },
                            "then": {
                                "type": "string",
                                "description": "Expected outcome"
                            }
                        }
                    },
                    "description": "Acceptance scenarios in Given/When/Then format"
                },
                "flow_diagram": {
                    "type": "string",
                    "description": "Optional Mermaid diagram code"
                },
                "data_model": {
                    "type": "object",
                    "description": "Optional JSON Schema for data model"
                }
            }
        }),
    }
}

/// Execute the create_spec tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    // Extract required fields
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;
    let title = get_required_string(args, "title")?;
    let overview = get_required_string(args, "overview")?;
    let requirements = get_required_array(args, "requirements")?;
    let scenarios = get_required_array(args, "scenarios")?;

    // Optional fields
    let flow_diagram = get_optional_string(args, "flow_diagram");
    let data_model = args.get("data_model").cloned();

    // Validate spec_id format
    if !spec_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("spec_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate overview length
    if overview.len() < 50 {
        anyhow::bail!("overview must be at least 50 characters");
    }

    // Validate requirements
    if requirements.is_empty() {
        anyhow::bail!("At least one requirement is required");
    }

    // Validate scenarios
    if scenarios.is_empty() {
        anyhow::bail!("At least one scenario is required");
    }

    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(&change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Create proposal first.",
            change_id
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
    content.push_str(&format!("id: {}\n", spec_id));
    content.push_str("type: spec\n");
    content.push_str(&format!("title: \"{}\"\n", title.replace('"', "\\\"")));
    content.push_str("version: 1\n");
    content.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));

    // Requirements summary
    let requirement_ids: Vec<String> = requirements
        .iter()
        .filter_map(|r| r.get("id").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .collect();

    content.push_str("requirements:\n");
    content.push_str(&format!("  total: {}\n", requirements.len()));
    if !requirement_ids.is_empty() {
        content.push_str("  ids:\n");
        for id in &requirement_ids {
            content.push_str(&format!("    - {}\n", id));
        }
    }

    // Design elements
    content.push_str("design_elements:\n");
    content.push_str(&format!("  has_mermaid: {}\n", flow_diagram.is_some()));
    content.push_str(&format!("  has_json_schema: {}\n", data_model.is_some()));
    content.push_str("  has_pseudo_code: false\n");
    content.push_str("  has_api_spec: false\n");

    content.push_str("---\n\n");

    // Wrap spec content in XML
    content.push_str("<spec>\n\n");

    // Title
    content.push_str(&format!("# {}\n\n", title));

    // Overview
    content.push_str("## Overview\n\n");
    content.push_str(&format!("{}\n\n", overview));

    // Requirements section
    content.push_str("## Requirements\n\n");

    for req in &requirements {
        let req_id = req
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("R?");
        let req_title = req
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled");
        let req_desc = req
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let priority = req
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        content.push_str(&format!("### {} - {}\n\n", req_id, req_title));
        content.push_str("```yaml\n");
        content.push_str(&format!("id: {}\n", req_id));
        content.push_str(&format!("priority: {}\n", priority));
        content.push_str("status: draft\n");
        content.push_str("```\n\n");
        content.push_str(&format!("{}\n\n", req_desc));
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

    for scenario in &scenarios {
        let name = scenario
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed Scenario");
        let given = scenario.get("given").and_then(|v| v.as_str());
        let when = scenario
            .get("when")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let then = scenario
            .get("then")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Use scenario heading format from rules: ### {prefix} {name}
        let heading_hashes = "#".repeat(spec_rules.scenario_heading_level as usize);
        content.push_str(&format!("{} {} {}\n\n", heading_hashes, spec_rules.scenario_heading_prefix, name));

        // Use WHEN/THEN keywords from rules
        if let Some(given_text) = given {
            content.push_str(&format!("- **GIVEN** {}\n", given_text));
        }
        content.push_str(&format!("- **{}** {}\n", spec_rules.when_keyword, when));
        content.push_str(&format!("- **{}** {}\n\n", spec_rules.then_keyword, then));
    }

    // Flow diagram (optional)
    if let Some(diagram) = flow_diagram {
        content.push_str("## Flow Diagram\n\n");
        content.push_str("```mermaid\n");
        content.push_str(&diagram);
        if !diagram.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("```\n\n");
    }

    // Data model (optional)
    if let Some(model) = data_model {
        content.push_str("## Data Model\n\n");
        content.push_str("```json\n");
        content.push_str(&serde_json::to_string_pretty(&model)?);
        content.push_str("\n```\n\n");
    }

    // Close spec XML tag
    content.push_str("</spec>\n");

    // Write the file
    let spec_path = specs_dir.join(format!("{}.md", spec_id));
    std::fs::write(&spec_path, &content)?;

    Ok(format!(
        "Created spec '{}' for change '{}' at {}",
        spec_id,
        change_id,
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

        let args = json!({
            "change_id": "test-change",
            "spec_id": "mcp-protocol",
            "title": "MCP Protocol Implementation",
            "overview": "This specification covers the implementation of the Model Context Protocol (MCP) server for agentd, providing structured tools for proposal generation.",
            "requirements": [
                {
                    "id": "R1",
                    "title": "JSON-RPC 2.0 Support",
                    "description": "The server must support JSON-RPC 2.0 protocol over stdio",
                    "priority": "high"
                },
                {
                    "id": "R2",
                    "title": "Tool Registration",
                    "description": "Tools must be registered and callable via tools/call method",
                    "priority": "high"
                }
            ],
            "scenarios": [
                {
                    "name": "Server Initialization",
                    "given": "MCP client is connected",
                    "when": "Client sends initialize request",
                    "then": "Server responds with capabilities"
                },
                {
                    "name": "Tool Execution",
                    "when": "Client calls create_proposal tool",
                    "then": "Server creates proposal.md and returns success"
                }
            ],
            "flow_diagram": "graph LR\n    A[Client] --> B[Server]\n    B --> C[Tool Registry]\n    C --> D[Execute Tool]"
        });

        let result = execute(&args, project_root).unwrap();
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
}
