//! create_spec MCP Tool
//!
//! Creates a validated spec file with requirements and acceptance criteria.

use super::{get_optional_string, get_required_array, get_required_string, ToolDefinition};
use crate::services::spec_service::{create_spec, CreateSpecInput, RequirementData, ScenarioData};
use crate::Result;
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
            "required": ["project_path", "change_id", "spec_id", "title", "overview", "requirements", "scenarios"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
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

    // Convert requirements JSON array to RequirementData
    let requirements_vec: Vec<RequirementData> = requirements
        .iter()
        .filter_map(|r| {
            Some(RequirementData {
                id: r.get("id")?.as_str()?.to_string(),
                title: r.get("title")?.as_str()?.to_string(),
                description: r.get("description")?.as_str()?.to_string(),
                priority: r
                    .get("priority")
                    .and_then(|p| p.as_str())
                    .unwrap_or("medium")
                    .to_string(),
            })
        })
        .collect();

    // Convert scenarios JSON array to ScenarioData
    let scenarios_vec: Vec<ScenarioData> = scenarios
        .iter()
        .filter_map(|s| {
            Some(ScenarioData {
                name: s.get("name")?.as_str()?.to_string(),
                given: s.get("given").and_then(|g| g.as_str()).map(String::from),
                when: s.get("when")?.as_str()?.to_string(),
                then: s.get("then")?.as_str()?.to_string(),
            })
        })
        .collect();

    // Create input struct and call service
    let input = CreateSpecInput {
        change_id,
        spec_id,
        title,
        overview,
        requirements: requirements_vec,
        scenarios: scenarios_vec,
        flow_diagram,
        data_model,
    };

    create_spec(input, project_root)
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
