//! generate_mermaid_flowchart MCP Tool
//!
//! Generates Mermaid flowchart diagrams for algorithms, business logic, and decision trees.

use super::super::{get_required_array, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_flowchart
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_flowchart".to_string(),
        description: "Generate a Mermaid flowchart diagram from structured node and edge definitions. Use for algorithms, business logic, decision trees, and process flows.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["direction", "nodes", "edges"],
            "properties": {
                "direction": {
                    "type": "string",
                    "enum": ["LR", "RL", "TB", "BT"],
                    "description": "Flow direction: LR (left-right), RL (right-left), TB (top-bottom), BT (bottom-top)"
                },
                "nodes": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["id", "label"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^[A-Za-z0-9_]+$",
                                "description": "Unique node identifier (alphanumeric + underscore)"
                            },
                            "label": {
                                "type": "string",
                                "description": "Node display text"
                            },
                            "shape": {
                                "type": "string",
                                "enum": ["rectangle", "rounded", "stadium", "subroutine", "cylinder", "circle", "diamond", "hexagon", "parallelogram", "trapezoid"],
                                "default": "rectangle",
                                "description": "Node shape"
                            }
                        }
                    },
                    "description": "List of nodes in the flowchart"
                },
                "edges": {
                    "type": "array",
                    "minItems": 0,
                    "items": {
                        "type": "object",
                        "required": ["from", "to"],
                        "properties": {
                            "from": {
                                "type": "string",
                                "description": "Source node ID"
                            },
                            "to": {
                                "type": "string",
                                "description": "Target node ID"
                            },
                            "label": {
                                "type": "string",
                                "description": "Edge label (optional)"
                            },
                            "style": {
                                "type": "string",
                                "enum": ["arrow", "thick", "dotted"],
                                "default": "arrow",
                                "description": "Edge style"
                            }
                        }
                    },
                    "description": "List of edges connecting nodes"
                },
                "subgraphs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "nodes"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Subgraph identifier"
                            },
                            "title": {
                                "type": "string",
                                "description": "Subgraph title"
                            },
                            "nodes": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Node IDs in this subgraph"
                            }
                        }
                    },
                    "description": "Optional subgraphs for grouping nodes"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_flowchart tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let direction = get_required_string(args, "direction")?;
    let nodes = get_required_array(args, "nodes")?;
    let edges = get_required_array(args, "edges")?;

    // Optional fields
    let subgraphs = args
        .get("subgraphs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Validate direction
    if !["LR", "RL", "TB", "BT"].contains(&direction.as_str()) {
        anyhow::bail!("Invalid direction. Must be one of: LR, RL, TB, BT");
    }

    // Validate nodes
    if nodes.is_empty() {
        anyhow::bail!("At least one node is required");
    }

    // Generate Mermaid flowchart
    let mut mermaid = String::new();
    mermaid.push_str(&format!("flowchart {}\n", direction));

    // Generate subgraphs first
    for subgraph in &subgraphs {
        let sg_id = subgraph
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Subgraph missing 'id' field"))?;
        let sg_title = subgraph
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Subgraph missing 'title' field"))?;
        let sg_nodes = subgraph
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Subgraph missing 'nodes' array"))?;

        mermaid.push_str(&format!("    subgraph {}[\"{}\"]\n", sg_id, sg_title));

        // Generate nodes within subgraph
        for node_id in sg_nodes {
            let node_id_str = node_id
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Subgraph node ID must be a string"))?;

            // Find the node definition
            let node = nodes
                .iter()
                .find(|n| n.get("id").and_then(|v| v.as_str()) == Some(node_id_str))
                .ok_or_else(|| {
                    anyhow::anyhow!("Node '{}' referenced in subgraph not found", node_id_str)
                })?;

            let label = node
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or(node_id_str);
            let shape = node
                .get("shape")
                .and_then(|v| v.as_str())
                .unwrap_or("rectangle");

            mermaid.push_str(&format!("        {}\n", format_node(node_id_str, label, shape)?));
        }

        mermaid.push_str("    end\n");
    }

    // Generate standalone nodes (not in subgraphs)
    let subgraph_node_ids: std::collections::HashSet<String> = subgraphs
        .iter()
        .filter_map(|sg| sg.get("nodes").and_then(|v| v.as_array()))
        .flat_map(|nodes| {
            nodes
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    for node in &nodes {
        let node_id = node
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Node missing 'id' field"))?;

        // Skip if already in a subgraph
        if subgraph_node_ids.contains(node_id) {
            continue;
        }

        let label = node
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or(node_id);
        let shape = node
            .get("shape")
            .and_then(|v| v.as_str())
            .unwrap_or("rectangle");

        mermaid.push_str(&format!("    {}\n", format_node(node_id, label, shape)?));
    }

    // Generate edges
    for edge in &edges {
        let from = edge
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Edge missing 'from' field"))?;
        let to = edge
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Edge missing 'to' field"))?;
        let label = edge.get("label").and_then(|v| v.as_str());
        let style = edge
            .get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("arrow");

        mermaid.push_str(&format!("    {}\n", format_edge(from, to, label, style)?));
    }

    Ok(mermaid)
}

/// Format a node based on its shape
fn format_node(id: &str, label: &str, shape: &str) -> Result<String> {
    let escaped_label = label.replace('"', "#quot;");

    let node_str = match shape {
        "rectangle" => format!("{}[{}]", id, escaped_label),
        "rounded" => format!("{}({})", id, escaped_label),
        "stadium" => format!("{}([{}])", id, escaped_label),
        "subroutine" => format!("{}[[{}]]", id, escaped_label),
        "cylinder" => format!("{}[({})]", id, escaped_label),
        "circle" => format!("{}(({}))", id, escaped_label),
        "diamond" => format!("{}{{{}}} ", id, escaped_label),
        "hexagon" => format!("{}{{{{{}}}}}", id, escaped_label),
        "parallelogram" => format!("{}[/{}\\]", id, escaped_label),
        "trapezoid" => format!("{}[\\{}/]", id, escaped_label),
        _ => anyhow::bail!("Invalid node shape: {}", shape),
    };

    Ok(node_str)
}

/// Format an edge based on its style
fn format_edge(from: &str, to: &str, label: Option<&str>, style: &str) -> Result<String> {
    let edge_str = match (style, label) {
        ("arrow", Some(lbl)) => format!("{} -->|{}| {}", from, lbl, to),
        ("arrow", None) => format!("{} --> {}", from, to),
        ("thick", Some(lbl)) => format!("{} ==>|{}| {}", from, lbl, to),
        ("thick", None) => format!("{} ==> {}", from, to),
        ("dotted", Some(lbl)) => format!("{} -.->|{}| {}", from, lbl, to),
        ("dotted", None) => format!("{} -.-> {}", from, to),
        _ => anyhow::bail!("Invalid edge style: {}", style),
    };

    Ok(edge_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_flowchart() {
        let args = json!({
            "direction": "LR",
            "nodes": [
                {"id": "A", "label": "Start", "shape": "rounded"},
                {"id": "B", "label": "Process", "shape": "rectangle"},
                {"id": "C", "label": "Decision", "shape": "diamond"},
                {"id": "D", "label": "End", "shape": "rounded"}
            ],
            "edges": [
                {"from": "A", "to": "B"},
                {"from": "B", "to": "C"},
                {"from": "C", "to": "D", "label": "Yes", "style": "arrow"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("flowchart LR"));
        assert!(result.contains("A(Start)"));
        assert!(result.contains("B[Process]"));
        assert!(result.contains("C{Decision}"));
        assert!(result.contains("D(End)"));
        assert!(result.contains("A --> B"));
        assert!(result.contains("B --> C"));
        assert!(result.contains("C -->|Yes| D"));
    }

    #[test]
    fn test_flowchart_with_subgraph() {
        let args = json!({
            "direction": "TB",
            "nodes": [
                {"id": "A", "label": "Start", "shape": "rounded"},
                {"id": "B", "label": "Task 1", "shape": "rectangle"},
                {"id": "C", "label": "Task 2", "shape": "rectangle"},
                {"id": "D", "label": "End", "shape": "rounded"}
            ],
            "edges": [
                {"from": "A", "to": "B"},
                {"from": "B", "to": "C"},
                {"from": "C", "to": "D"}
            ],
            "subgraphs": [
                {
                    "id": "sub1",
                    "title": "Processing",
                    "nodes": ["B", "C"]
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("flowchart TB"));
        assert!(result.contains("subgraph sub1[\"Processing\"]"));
        assert!(result.contains("A(Start)"));
        assert!(result.contains("D(End)"));
    }

    #[test]
    fn test_invalid_direction() {
        let args = json!({
            "direction": "INVALID",
            "nodes": [{"id": "A", "label": "Test"}],
            "edges": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid direction"));
    }

    #[test]
    fn test_empty_nodes() {
        let args = json!({
            "direction": "LR",
            "nodes": [],
            "edges": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one node is required"));
    }
}
