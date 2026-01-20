//! generate_mermaid_class MCP Tool
//!
//! Generates Mermaid class diagrams for data structures, domain models, and OOP design.

use super::super::{get_required_array, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_class
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_class".to_string(),
        description: "Generate a Mermaid class diagram from structured class and relationship definitions. Use for data structures, domain models, and object-oriented design.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["classes"],
            "properties": {
                "classes": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Class name"
                            },
                            "stereotype": {
                                "type": "string",
                                "enum": ["interface", "abstract", "enumeration", "service"],
                                "description": "Class stereotype"
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
                                            "description": "Attribute type"
                                        },
                                        "visibility": {
                                            "type": "string",
                                            "enum": ["public", "private", "protected", "package"],
                                            "default": "public",
                                            "description": "Attribute visibility"
                                        },
                                        "static": {
                                            "type": "boolean",
                                            "default": false,
                                            "description": "Is static attribute"
                                        }
                                    }
                                },
                                "description": "Class attributes/fields"
                            },
                            "methods": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "required": ["name"],
                                    "properties": {
                                        "name": {
                                            "type": "string",
                                            "description": "Method name"
                                        },
                                        "parameters": {
                                            "type": "array",
                                            "items": {
                                                "type": "object",
                                                "required": ["name", "type"],
                                                "properties": {
                                                    "name": {"type": "string"},
                                                    "type": {"type": "string"}
                                                }
                                            },
                                            "description": "Method parameters"
                                        },
                                        "return_type": {
                                            "type": "string",
                                            "description": "Return type"
                                        },
                                        "visibility": {
                                            "type": "string",
                                            "enum": ["public", "private", "protected", "package"],
                                            "default": "public",
                                            "description": "Method visibility"
                                        },
                                        "static": {
                                            "type": "boolean",
                                            "default": false,
                                            "description": "Is static method"
                                        },
                                        "abstract": {
                                            "type": "boolean",
                                            "default": false,
                                            "description": "Is abstract method"
                                        }
                                    }
                                },
                                "description": "Class methods"
                            }
                        }
                    },
                    "description": "List of classes"
                },
                "relationships": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["from", "to", "type"],
                        "properties": {
                            "from": {
                                "type": "string",
                                "description": "Source class name"
                            },
                            "to": {
                                "type": "string",
                                "description": "Target class name"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["inheritance", "composition", "aggregation", "association", "dependency", "realization"],
                                "description": "Relationship type"
                            },
                            "label": {
                                "type": "string",
                                "description": "Relationship label"
                            },
                            "multiplicity_from": {
                                "type": "string",
                                "description": "Multiplicity at source (e.g., '1', '0..1', '1..*')"
                            },
                            "multiplicity_to": {
                                "type": "string",
                                "description": "Multiplicity at target"
                            }
                        }
                    },
                    "description": "List of relationships between classes"
                },
                "namespaces": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["name", "classes"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Namespace/package name"
                            },
                            "classes": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Class names in this namespace"
                            }
                        }
                    },
                    "description": "Optional namespaces/packages"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_class tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let classes = get_required_array(args, "classes")?;

    // Optional fields
    let relationships = args
        .get("relationships")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let namespaces = args
        .get("namespaces")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Validate
    if classes.is_empty() {
        anyhow::bail!("At least one class is required");
    }

    // Generate Mermaid class diagram
    let mut mermaid = String::new();
    mermaid.push_str("classDiagram\n");

    // Track which classes are in namespaces
    let namespace_classes: std::collections::HashSet<String> = namespaces
        .iter()
        .filter_map(|ns| ns.get("classes").and_then(|v| v.as_array()))
        .flat_map(|classes_array| {
            classes_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    // Generate namespaces
    for namespace in &namespaces {
        let ns_name = namespace
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Namespace missing 'name' field"))?;
        let ns_classes = namespace
            .get("classes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Namespace missing 'classes' array"))?;

        mermaid.push_str(&format!("    namespace {} {{\n", ns_name));

        // Generate classes within namespace
        for class_name in ns_classes {
            let class_name_str = class_name
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Class name must be a string"))?;

            // Find the class definition
            let class = classes
                .iter()
                .find(|c| c.get("name").and_then(|v| v.as_str()) == Some(class_name_str))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Class '{}' referenced in namespace not found",
                        class_name_str
                    )
                })?;

            mermaid.push_str(&format_class(class, "        ")?);
        }

        mermaid.push_str("    }\n");
    }

    // Generate standalone classes (not in namespaces)
    for class in &classes {
        let class_name = class
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Class missing 'name' field"))?;

        // Skip if already in a namespace
        if namespace_classes.contains(class_name) {
            continue;
        }

        mermaid.push_str(&format_class(class, "    ")?);
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
        let label = rel.get("label").and_then(|v| v.as_str());
        let mult_from = rel.get("multiplicity_from").and_then(|v| v.as_str());
        let mult_to = rel.get("multiplicity_to").and_then(|v| v.as_str());

        mermaid.push_str(&format_relationship(
            from, to, rel_type, label, mult_from, mult_to,
        )?);
    }

    Ok(mermaid)
}

/// Format a class definition
fn format_class(class: &Value, indent: &str) -> Result<String> {
    let name = class
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Class missing 'name' field"))?;

    let mut class_str = String::new();

    // Add stereotype if present
    if let Some(stereotype) = class.get("stereotype").and_then(|v| v.as_str()) {
        class_str.push_str(&format!("{}class {} {{\n", indent, name));
        class_str.push_str(&format!("{}    <<{}>>\n", indent, stereotype));
    } else {
        class_str.push_str(&format!("{}class {} {{\n", indent, name));
    }

    // Add attributes
    if let Some(attributes) = class.get("attributes").and_then(|v| v.as_array()) {
        for attr in attributes {
            let attr_name = attr
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Attribute missing 'name' field"))?;
            let attr_type = attr
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Attribute missing 'type' field"))?;
            let visibility = attr
                .get("visibility")
                .and_then(|v| v.as_str())
                .unwrap_or("public");
            let is_static = attr
                .get("static")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let vis_symbol = match visibility {
                "public" => "+",
                "private" => "-",
                "protected" => "#",
                "package" => "~",
                _ => "+",
            };

            let static_marker = if is_static { "$" } else { "" };

            class_str.push_str(&format!(
                "{}    {}{}{} {}\n",
                indent, vis_symbol, static_marker, attr_name, attr_type
            ));
        }
    }

    // Add methods
    if let Some(methods) = class.get("methods").and_then(|v| v.as_array()) {
        for method in methods {
            let method_name = method
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Method missing 'name' field"))?;
            let visibility = method
                .get("visibility")
                .and_then(|v| v.as_str())
                .unwrap_or("public");
            let is_static = method
                .get("static")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let is_abstract = method
                .get("abstract")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let vis_symbol = match visibility {
                "public" => "+",
                "private" => "-",
                "protected" => "#",
                "package" => "~",
                _ => "+",
            };

            let static_marker = if is_static { "$" } else { "" };
            let abstract_marker = if is_abstract { "*" } else { "" };

            // Format parameters
            let params = if let Some(parameters) = method.get("parameters").and_then(|v| v.as_array())
            {
                parameters
                    .iter()
                    .filter_map(|p| {
                        let param_name = p.get("name").and_then(|v| v.as_str())?;
                        let param_type = p.get("type").and_then(|v| v.as_str())?;
                        Some(format!("{} {}", param_name, param_type))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                String::new()
            };

            // Format return type
            let return_type = method
                .get("return_type")
                .and_then(|v| v.as_str())
                .unwrap_or("void");

            class_str.push_str(&format!(
                "{}    {}{}{}{}({}) {}\n",
                indent, vis_symbol, static_marker, abstract_marker, method_name, params, return_type
            ));
        }
    }

    class_str.push_str(&format!("{}}}\n", indent));

    Ok(class_str)
}

/// Format a relationship
fn format_relationship(
    from: &str,
    to: &str,
    rel_type: &str,
    label: Option<&str>,
    mult_from: Option<&str>,
    mult_to: Option<&str>,
) -> Result<String> {
    let arrow = match rel_type {
        "inheritance" => "<|--",
        "composition" => "*--",
        "aggregation" => "o--",
        "association" => "-->",
        "dependency" => "..>",
        "realization" => "..|>",
        _ => anyhow::bail!("Invalid relationship type: {}", rel_type),
    };

    let mut rel_str = format!("    {} {} {}", from, arrow, to);

    // Add label if present
    if let Some(lbl) = label {
        rel_str.push_str(&format!(" : {}", lbl));
    }

    // Add multiplicities if present
    if mult_from.is_some() || mult_to.is_some() {
        rel_str.push('\n');
        if let Some(m_from) = mult_from {
            rel_str.push_str(&format!("    {} \"{}\" {}\n", from, m_from, to));
        }
        if let Some(m_to) = mult_to {
            rel_str.push_str(&format!("    {} {} \"{}\"\n", from, to, m_to));
        }
    } else {
        rel_str.push('\n');
    }

    Ok(rel_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_class_diagram() {
        let args = json!({
            "classes": [
                {
                    "name": "Animal",
                    "attributes": [
                        {"name": "name", "type": "string", "visibility": "private"}
                    ],
                    "methods": [
                        {"name": "speak", "return_type": "void", "visibility": "public"}
                    ]
                },
                {
                    "name": "Dog",
                    "methods": [
                        {"name": "bark", "return_type": "void"}
                    ]
                }
            ],
            "relationships": [
                {"from": "Dog", "to": "Animal", "type": "inheritance"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("classDiagram"));
        assert!(result.contains("class Animal"));
        assert!(result.contains("-name string"));
        assert!(result.contains("+speak() void"));
        assert!(result.contains("Dog <|-- Animal"));
    }

    #[test]
    fn test_class_with_stereotype() {
        let args = json!({
            "classes": [
                {
                    "name": "Serializable",
                    "stereotype": "interface",
                    "methods": [
                        {"name": "serialize", "return_type": "string", "abstract": true}
                    ]
                }
            ],
            "relationships": []
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("<<interface>>"));
        assert!(result.contains("+*serialize() string"));
    }

    #[test]
    fn test_empty_classes() {
        let args = json!({
            "classes": [],
            "relationships": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one class is required"));
    }
}
