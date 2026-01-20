//! generate_mermaid_requirement MCP Tool
//!
//! Generates Mermaid requirement diagrams for requirement analysis and verification relationships.

use super::super::{get_required_array, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_requirement
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_requirement".to_string(),
        description: "Generate a Mermaid requirement diagram from structured requirement and element definitions. Use for requirement traceability, verification relationships, and requirement analysis.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["requirements"],
            "properties": {
                "requirements": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["id", "text", "risk", "verification"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Requirement identifier (e.g., R1, REQ-001)"
                            },
                            "text": {
                                "type": "string",
                                "description": "Requirement text/description"
                            },
                            "risk": {
                                "type": "string",
                                "enum": ["Low", "Medium", "High"],
                                "description": "Requirement risk level"
                            },
                            "verification": {
                                "type": "string",
                                "enum": ["Analysis", "Inspection", "Test", "Demonstration"],
                                "description": "Verification method"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["requirement", "functionalRequirement", "interfaceRequirement", "performanceRequirement", "physicalRequirement", "designConstraint"],
                                "default": "requirement",
                                "description": "Requirement type"
                            }
                        }
                    },
                    "description": "List of requirements"
                },
                "elements": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "text", "type", "docref"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Element identifier"
                            },
                            "text": {
                                "type": "string",
                                "description": "Element text/name"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["element"],
                                "default": "element",
                                "description": "Element type"
                            },
                            "docref": {
                                "type": "string",
                                "description": "Document reference"
                            }
                        }
                    },
                    "description": "Optional design elements"
                },
                "relationships": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["from", "to", "type"],
                        "properties": {
                            "from": {
                                "type": "string",
                                "description": "Source requirement/element ID"
                            },
                            "to": {
                                "type": "string",
                                "description": "Target requirement/element ID"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["satisfies", "verifies", "refines", "traces", "contains", "copies", "derives"],
                                "description": "Relationship type"
                            }
                        }
                    },
                    "description": "Optional relationships between requirements/elements"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_requirement tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let requirements = get_required_array(args, "requirements")?;

    // Optional fields
    let elements = args
        .get("elements")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let relationships = args
        .get("relationships")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Validate
    if requirements.is_empty() {
        anyhow::bail!("At least one requirement is required");
    }

    // Generate Mermaid requirement diagram
    let mut mermaid = String::new();
    mermaid.push_str("requirementDiagram\n\n");

    // Generate requirements
    for req in &requirements {
        let req_id = req
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Requirement missing 'id' field"))?;
        let req_text = req
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Requirement missing 'text' field"))?;
        let risk = req
            .get("risk")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Requirement missing 'risk' field"))?;
        let verification = req
            .get("verification")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Requirement missing 'verification' field"))?;
        let req_type = req
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("requirement");

        // Validate risk and verification
        if !["Low", "Medium", "High"].contains(&risk) {
            anyhow::bail!("Invalid risk level: {}", risk);
        }
        if !["Analysis", "Inspection", "Test", "Demonstration"].contains(&verification) {
            anyhow::bail!("Invalid verification method: {}", verification);
        }

        mermaid.push_str(&format!("    {} {} {{\n", req_type, req_id));
        mermaid.push_str(&format!("        id: {}\n", req_id));
        mermaid.push_str(&format!("        text: {}\n", req_text));
        mermaid.push_str(&format!("        risk: {}\n", risk));
        mermaid.push_str(&format!("        verifymethod: {}\n", verification));
        mermaid.push_str("    }\n\n");
    }

    // Generate elements
    for elem in &elements {
        let elem_id = elem
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Element missing 'id' field"))?;
        let elem_text = elem
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Element missing 'text' field"))?;
        let elem_type = elem
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("element");
        let docref = elem
            .get("docref")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Element missing 'docref' field"))?;

        mermaid.push_str(&format!("    {} {} {{\n", elem_type, elem_id));
        mermaid.push_str(&format!("        type: {}\n", elem_text));
        mermaid.push_str(&format!("        docref: {}\n", docref));
        mermaid.push_str("    }\n\n");
    }

    // Generate relationships
    for rel in &relationships {
        let from = rel
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Relationship missing 'from' field"))?;
        let to = rel
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Relationship missing 'to' field"))?;
        let rel_type = rel
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Relationship missing 'type' field"))?;

        // Validate relationship type
        if !["satisfies", "verifies", "refines", "traces", "contains", "copies", "derives"]
            .contains(&rel_type)
        {
            anyhow::bail!("Invalid relationship type: {}", rel_type);
        }

        mermaid.push_str(&format!("    {} - {} -> {}\n", from, rel_type, to));
    }

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_requirement_diagram() {
        let args = json!({
            "requirements": [
                {
                    "id": "R1",
                    "text": "System shall authenticate users",
                    "risk": "High",
                    "verification": "Test",
                    "type": "functionalRequirement"
                },
                {
                    "id": "R2",
                    "text": "Response time < 100ms",
                    "risk": "Medium",
                    "verification": "Test",
                    "type": "performanceRequirement"
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("requirementDiagram"));
        assert!(result.contains("functionalRequirement R1"));
        assert!(result.contains("text: System shall authenticate users"));
        assert!(result.contains("risk: High"));
        assert!(result.contains("verifymethod: Test"));
        assert!(result.contains("performanceRequirement R2"));
    }

    #[test]
    fn test_requirement_with_elements() {
        let args = json!({
            "requirements": [
                {
                    "id": "R1",
                    "text": "User login",
                    "risk": "Low",
                    "verification": "Analysis"
                }
            ],
            "elements": [
                {
                    "id": "LoginModule",
                    "text": "Login Module",
                    "type": "element",
                    "docref": "docs/login.md"
                }
            ],
            "relationships": [
                {
                    "from": "LoginModule",
                    "to": "R1",
                    "type": "satisfies"
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("element LoginModule"));
        assert!(result.contains("docref: docs/login.md"));
        assert!(result.contains("LoginModule - satisfies -> R1"));
    }

    #[test]
    fn test_invalid_risk() {
        let args = json!({
            "requirements": [
                {
                    "id": "R1",
                    "text": "Test",
                    "risk": "Invalid",
                    "verification": "Test"
                }
            ]
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid risk level"));
    }

    #[test]
    fn test_invalid_verification() {
        let args = json!({
            "requirements": [
                {
                    "id": "R1",
                    "text": "Test",
                    "risk": "Low",
                    "verification": "Invalid"
                }
            ]
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid verification method"));
    }

    #[test]
    fn test_empty_requirements() {
        let args = json!({
            "requirements": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one requirement is required"));
    }
}
