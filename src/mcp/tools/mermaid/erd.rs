//! generate_mermaid_erd MCP Tool
//!
//! Generates Mermaid Entity Relationship Diagrams for database design and data models.

use super::super::{get_required_array, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_erd
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_erd".to_string(),
        description: "Generate a Mermaid Entity Relationship Diagram from structured entity and relationship definitions. Use for database design, data modeling, and schema visualization.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["entities", "relationships"],
            "properties": {
                "entities": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Entity name"
                            },
                            "attributes": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "required": ["name", "type"],
                                    "properties": {
                                        "name": {
                                            "type": "string",
                                            "description": "Attribute name"
                                        },
                                        "type": {
                                            "type": "string",
                                            "description": "Attribute type (e.g., int, string, date)"
                                        },
                                        "key": {
                                            "type": "string",
                                            "enum": ["PK", "FK", "UK"],
                                            "description": "Key type: PK (Primary Key), FK (Foreign Key), UK (Unique Key)"
                                        },
                                        "nullable": {
                                            "type": "boolean",
                                            "default": true,
                                            "description": "Is nullable"
                                        },
                                        "comment": {
                                            "type": "string",
                                            "description": "Attribute comment/description"
                                        }
                                    }
                                },
                                "description": "Entity attributes/columns"
                            }
                        }
                    },
                    "description": "List of entities/tables"
                },
                "relationships": {
                    "type": "array",
                    "minItems": 0,
                    "items": {
                        "type": "object",
                        "required": ["from", "to", "cardinality"],
                        "properties": {
                            "from": {
                                "type": "string",
                                "description": "Source entity name"
                            },
                            "to": {
                                "type": "string",
                                "description": "Target entity name"
                            },
                            "cardinality": {
                                "type": "string",
                                "enum": [
                                    "one-to-one",
                                    "one-to-many",
                                    "many-to-one",
                                    "many-to-many",
                                    "one-or-more-to-one",
                                    "one-to-one-or-more",
                                    "zero-or-one-to-one",
                                    "one-to-zero-or-one"
                                ],
                                "description": "Relationship cardinality"
                            },
                            "identifying": {
                                "type": "boolean",
                                "default": false,
                                "description": "Is identifying relationship"
                            },
                            "label": {
                                "type": "string",
                                "description": "Relationship label"
                            }
                        }
                    },
                    "description": "List of relationships between entities"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_erd tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let entities = get_required_array(args, "entities")?;
    let relationships = get_required_array(args, "relationships")?;

    // Validate
    if entities.is_empty() {
        anyhow::bail!("At least one entity is required");
    }

    // Generate Mermaid ERD
    let mut mermaid = String::new();
    mermaid.push_str("erDiagram\n");

    // Generate entities
    for entity in &entities {
        let entity_name = entity
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Entity missing 'name' field"))?;

        mermaid.push_str(&format!("    {} {{\n", entity_name));

        // Generate attributes
        if let Some(attributes) = entity.get("attributes").and_then(|v| v.as_array()) {
            for attr in attributes {
                let attr_name = attr
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Attribute missing 'name' field"))?;
                let attr_type = attr
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Attribute missing 'type' field"))?;
                let key = attr.get("key").and_then(|v| v.as_str());
                let nullable = attr
                    .get("nullable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let comment = attr.get("comment").and_then(|v| v.as_str());

                // Format: type attribute_name KEY "comment"
                let mut attr_str = format!("        {} {}", attr_type, attr_name);

                if let Some(k) = key {
                    attr_str.push_str(&format!(" {}", k));
                }

                if !nullable {
                    // Mermaid doesn't have direct NOT NULL, use comment
                    if let Some(c) = comment {
                        attr_str.push_str(&format!(" \"NOT NULL, {}\"", c));
                    } else {
                        attr_str.push_str(" \"NOT NULL\"");
                    }
                } else if let Some(c) = comment {
                    attr_str.push_str(&format!(" \"{}\"", c));
                }

                mermaid.push_str(&format!("{}\n", attr_str));
            }
        }

        mermaid.push_str("    }\n");
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
        let cardinality = rel
            .get("cardinality")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Relationship missing 'cardinality' field"))?;
        let identifying = rel
            .get("identifying")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let label = rel.get("label").and_then(|v| v.as_str());

        let (left_card, right_card) = parse_cardinality(cardinality)?;

        // Format relationship
        let mut rel_str = format!("    {} {}{}{} {}", from, left_card,
            if identifying { "--" } else { ".." }, right_card, to);

        if let Some(lbl) = label {
            rel_str.push_str(&format!(" : \"{}\"", lbl));
        }

        mermaid.push_str(&format!("{}\n", rel_str));
    }

    Ok(mermaid)
}

/// Parse cardinality into Mermaid notation
fn parse_cardinality(cardinality: &str) -> Result<(&'static str, &'static str)> {
    let (left, right) = match cardinality {
        "one-to-one" => ("||", "||"),
        "one-to-many" => ("||", "}o"),
        "many-to-one" => ("}o", "||"),
        "many-to-many" => ("}o", "}o"),
        "one-or-more-to-one" => ("}|", "||"),
        "one-to-one-or-more" => ("||", "|{"),
        "zero-or-one-to-one" => ("|o", "||"),
        "one-to-zero-or-one" => ("||", "o|"),
        _ => anyhow::bail!("Invalid cardinality: {}", cardinality),
    };

    Ok((left, right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_erd() {
        let args = json!({
            "entities": [
                {
                    "name": "User",
                    "attributes": [
                        {"name": "id", "type": "int", "key": "PK", "nullable": false},
                        {"name": "email", "type": "string", "key": "UK"},
                        {"name": "name", "type": "string"}
                    ]
                },
                {
                    "name": "Post",
                    "attributes": [
                        {"name": "id", "type": "int", "key": "PK"},
                        {"name": "user_id", "type": "int", "key": "FK"},
                        {"name": "title", "type": "string"}
                    ]
                }
            ],
            "relationships": [
                {
                    "from": "User",
                    "to": "Post",
                    "cardinality": "one-to-many",
                    "label": "creates"
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("erDiagram"));
        assert!(result.contains("User {"));
        assert!(result.contains("int id PK"));
        assert!(result.contains("Post {"));
        assert!(result.contains("User ||..}o Post"));
        assert!(result.contains("\"creates\""));
    }

    #[test]
    fn test_many_to_many() {
        let args = json!({
            "entities": [
                {
                    "name": "Student",
                    "attributes": [
                        {"name": "id", "type": "int", "key": "PK"}
                    ]
                },
                {
                    "name": "Course",
                    "attributes": [
                        {"name": "id", "type": "int", "key": "PK"}
                    ]
                }
            ],
            "relationships": [
                {
                    "from": "Student",
                    "to": "Course",
                    "cardinality": "many-to-many",
                    "label": "enrolls"
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("Student }o..}o Course"));
    }

    #[test]
    fn test_identifying_relationship() {
        let args = json!({
            "entities": [
                {
                    "name": "Order",
                    "attributes": [{"name": "id", "type": "int", "key": "PK"}]
                },
                {
                    "name": "OrderItem",
                    "attributes": [
                        {"name": "order_id", "type": "int", "key": "PK,FK"},
                        {"name": "item_id", "type": "int", "key": "PK"}
                    ]
                }
            ],
            "relationships": [
                {
                    "from": "Order",
                    "to": "OrderItem",
                    "cardinality": "one-to-many",
                    "identifying": true
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("Order ||--}o OrderItem"));
    }

    #[test]
    fn test_empty_entities() {
        let args = json!({
            "entities": [],
            "relationships": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one entity is required"));
    }
}
