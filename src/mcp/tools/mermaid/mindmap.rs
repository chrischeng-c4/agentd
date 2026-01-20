//! generate_mermaid_mindmap MCP Tool
//!
//! Generates Mermaid mindmap diagrams for concept organization and feature breakdown.

use super::super::ToolDefinition;
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_mindmap
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_mindmap".to_string(),
        description: "Generate a Mermaid mindmap diagram from structured node definitions. Use for brainstorming, concept organization, feature breakdown, and hierarchical thinking.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["root", "nodes"],
            "properties": {
                "root": {
                    "type": "object",
                    "required": ["label"],
                    "properties": {
                        "label": {
                            "type": "string",
                            "description": "Root node label"
                        },
                        "shape": {
                            "type": "string",
                            "enum": ["square", "rounded", "circle", "bang", "cloud", "hexagon"],
                            "default": "square",
                            "description": "Root node shape"
                        },
                        "icon": {
                            "type": "string",
                            "description": "Optional icon (emoji or text)"
                        }
                    },
                    "description": "Root node of the mindmap"
                },
                "nodes": {
                    "type": "array",
                    "minItems": 0,
                    "items": {
                        "type": "object",
                        "required": ["parent", "label"],
                        "properties": {
                            "parent": {
                                "type": "string",
                                "description": "Parent node label"
                            },
                            "label": {
                                "type": "string",
                                "description": "Node label"
                            },
                            "shape": {
                                "type": "string",
                                "enum": ["square", "rounded", "circle", "bang", "cloud", "hexagon"],
                                "default": "square",
                                "description": "Node shape"
                            },
                            "icon": {
                                "type": "string",
                                "description": "Optional icon (emoji or text)"
                            }
                        }
                    },
                    "description": "Child nodes in the mindmap"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_mindmap tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let root = args
        .get("root")
        .ok_or_else(|| anyhow::anyhow!("Missing 'root' field"))?;
    let nodes = args
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let root_label = root
        .get("label")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Root missing 'label' field"))?;
    let root_shape = root.get("shape").and_then(|v| v.as_str()).unwrap_or("square");
    let root_icon = root.get("icon").and_then(|v| v.as_str());

    // Build a tree structure
    let mut tree: std::collections::HashMap<String, Vec<&Value>> =
        std::collections::HashMap::new();

    for node in &nodes {
        let parent = node
            .get("parent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Node missing 'parent' field"))?;

        tree.entry(parent.to_string())
            .or_insert_with(Vec::new)
            .push(node);
    }

    // Generate Mermaid mindmap
    let mut mermaid = String::new();
    mermaid.push_str("mindmap\n");

    // Format root
    mermaid.push_str(&format!(
        "  {}\n",
        format_node(root_label, root_shape, root_icon)?
    ));

    // Recursively generate child nodes
    generate_children(&mut mermaid, root_label, &tree, 2)?;

    Ok(mermaid)
}

/// Recursively generate child nodes
fn generate_children(
    mermaid: &mut String,
    parent_label: &str,
    tree: &std::collections::HashMap<String, Vec<&Value>>,
    indent_level: usize,
) -> Result<()> {
    if let Some(children) = tree.get(parent_label) {
        for child in children {
            let child_label = child
                .get("label")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Child node missing 'label' field"))?;
            let child_shape = child
                .get("shape")
                .and_then(|v| v.as_str())
                .unwrap_or("square");
            let child_icon = child.get("icon").and_then(|v| v.as_str());

            let indent = "  ".repeat(indent_level);
            mermaid.push_str(&format!(
                "{}{}\n",
                indent,
                format_node(child_label, child_shape, child_icon)?
            ));

            // Recursively generate grandchildren
            generate_children(mermaid, child_label, tree, indent_level + 1)?;
        }
    }

    Ok(())
}

/// Format a node based on its shape
fn format_node(label: &str, shape: &str, icon: Option<&str>) -> Result<String> {
    let display_label = if let Some(ic) = icon {
        format!("{} {}", ic, label)
    } else {
        label.to_string()
    };

    let node_str = match shape {
        "square" => format!("[{}]", display_label),
        "rounded" => format!("({})", display_label),
        "circle" => format!("(({}))", display_label),
        "bang" => format!(")){}((", display_label),
        "cloud" => format!("){}(", display_label),
        "hexagon" => format!("{{{{{}}}}}", display_label),
        _ => anyhow::bail!("Invalid node shape: {}", shape),
    };

    Ok(node_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mindmap() {
        let args = json!({
            "root": {
                "label": "Project",
                "shape": "square"
            },
            "nodes": [
                {"parent": "Project", "label": "Frontend", "shape": "rounded"},
                {"parent": "Project", "label": "Backend", "shape": "rounded"},
                {"parent": "Frontend", "label": "React", "shape": "circle"},
                {"parent": "Frontend", "label": "Vue", "shape": "circle"},
                {"parent": "Backend", "label": "Node.js", "shape": "circle"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("mindmap"));
        assert!(result.contains("[Project]"));
        assert!(result.contains("(Frontend)"));
        assert!(result.contains("(Backend)"));
        assert!(result.contains("((React))"));
        assert!(result.contains("((Vue))"));
        assert!(result.contains("((Node.js))"));
    }

    #[test]
    fn test_mindmap_with_icons() {
        let args = json!({
            "root": {
                "label": "Features",
                "shape": "rounded",
                "icon": "🎯"
            },
            "nodes": [
                {"parent": "Features", "label": "Authentication", "icon": "🔐"},
                {"parent": "Features", "label": "Dashboard", "icon": "📊"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("🎯 Features"));
        assert!(result.contains("🔐 Authentication"));
        assert!(result.contains("📊 Dashboard"));
    }

    #[test]
    fn test_mindmap_shapes() {
        let args = json!({
            "root": {
                "label": "Root",
                "shape": "hexagon"
            },
            "nodes": [
                {"parent": "Root", "label": "Bang", "shape": "bang"},
                {"parent": "Root", "label": "Cloud", "shape": "cloud"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("{{{{Root}}}}"));
        assert!(result.contains("))Bang(("));
        assert!(result.contains(")Cloud("));
    }

    #[test]
    fn test_missing_root() {
        let args = json!({
            "nodes": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'root' field"));
    }
}
