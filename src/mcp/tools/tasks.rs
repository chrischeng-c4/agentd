//! create_tasks MCP Tool
//!
//! Creates a validated tasks.md file with layered task structure.

use super::{get_required_array, get_required_string, ToolDefinition};
use crate::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// Get the tool definition for create_tasks
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_tasks".to_string(),
        description: "Create a validated tasks.md file with layered task structure".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id", "tasks"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID these tasks belong to"
                },
                "tasks": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["layer", "number", "title", "file", "spec_ref", "description"],
                        "properties": {
                            "layer": {
                                "enum": ["data", "logic", "integration", "testing"],
                                "description": "Task layer (build order)"
                            },
                            "number": {
                                "type": "integer",
                                "minimum": 1,
                                "description": "Task number within the layer"
                            },
                            "title": {
                                "type": "string",
                                "description": "Short task title"
                            },
                            "file": {
                                "type": "object",
                                "required": ["path", "action"],
                                "properties": {
                                    "path": {
                                        "type": "string",
                                        "description": "File path relative to project root"
                                    },
                                    "action": {
                                        "enum": ["CREATE", "MODIFY", "DELETE"],
                                        "description": "File action type"
                                    }
                                }
                            },
                            "spec_ref": {
                                "type": "string",
                                "description": "Reference to requirement (e.g., 'mcp-spec:R1')"
                            },
                            "description": {
                                "type": "string",
                                "description": "Detailed task description"
                            },
                            "depends": {
                                "type": "array",
                                "items": { "type": "string" },
                                "default": [],
                                "description": "Task IDs this task depends on (e.g., ['1.1', '1.2'])"
                            }
                        }
                    },
                    "description": "List of implementation tasks"
                }
            }
        }),
    }
}

/// Layer number for ordering
fn layer_number(layer: &str) -> u32 {
    match layer {
        "data" => 1,
        "logic" => 2,
        "integration" => 3,
        "testing" => 4,
        _ => 5,
    }
}

/// Layer display name
fn layer_display_name(layer: &str) -> &str {
    match layer {
        "data" => "Data Layer",
        "logic" => "Logic Layer",
        "integration" => "Integration Layer",
        "testing" => "Testing Layer",
        _ => "Other",
    }
}

/// Execute the create_tasks tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    // Extract required fields
    let change_id = get_required_string(args, "change_id")?;
    let tasks = get_required_array(args, "tasks")?;

    // Validate tasks
    if tasks.is_empty() {
        anyhow::bail!("At least one task is required");
    }

    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(&change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Create proposal first.",
            change_id
        );
    }

    // Group tasks by layer
    let mut tasks_by_layer: HashMap<String, Vec<&Value>> = HashMap::new();
    for task in &tasks {
        let layer = task
            .get("layer")
            .and_then(|v| v.as_str())
            .unwrap_or("logic")
            .to_string();
        tasks_by_layer
            .entry(layer)
            .or_default()
            .push(task);
    }

    // Sort layers by layer number
    let mut sorted_layers: Vec<_> = tasks_by_layer.keys().cloned().collect();
    sorted_layers.sort_by_key(|l| layer_number(l));

    // Calculate summary stats
    let total_tasks = tasks.len();
    let mut layer_stats: HashMap<String, (u32, u32)> = HashMap::new(); // (task_count, est_files)
    for (layer, layer_tasks) in &tasks_by_layer {
        let task_count = layer_tasks.len() as u32;
        let est_files = layer_tasks
            .iter()
            .filter(|t| {
                t.get("file")
                    .and_then(|f| f.get("action"))
                    .and_then(|a| a.as_str())
                    .map(|a| a == "CREATE")
                    .unwrap_or(false)
            })
            .count() as u32;
        layer_stats.insert(layer.clone(), (task_count, est_files));
    }

    // Generate tasks.md content
    let now = Utc::now();
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("id: {}\n", change_id));
    content.push_str("type: tasks\n");
    content.push_str("version: 1\n");
    content.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("proposal_ref: {}\n", change_id));

    // Summary
    content.push_str("summary:\n");
    content.push_str(&format!("  total: {}\n", total_tasks));
    content.push_str("  completed: 0\n");
    content.push_str("  in_progress: 0\n");
    content.push_str("  blocked: 0\n");
    content.push_str(&format!("  pending: {}\n", total_tasks));

    // Layers breakdown
    content.push_str("layers:\n");
    for layer in &sorted_layers {
        if let Some((task_count, est_files)) = layer_stats.get(layer) {
            content.push_str(&format!("  {}:\n", layer));
            content.push_str(&format!("    task_count: {}\n", task_count));
            content.push_str(&format!("    estimated_files: {}\n", est_files));
        }
    }

    content.push_str("---\n\n");

    // Title
    content.push_str("# Implementation Tasks\n\n");

    // Overview
    content.push_str("## Overview\n\n");
    content.push_str(&format!(
        "This document outlines {} implementation tasks for change `{}`.\n\n",
        total_tasks, change_id
    ));

    // Task summary table
    content.push_str("| Layer | Tasks | Status |\n");
    content.push_str("|-------|-------|--------|\n");
    for layer in &sorted_layers {
        if let Some((task_count, _)) = layer_stats.get(layer) {
            content.push_str(&format!(
                "| {} | {} | 🔲 Pending |\n",
                layer_display_name(layer),
                task_count
            ));
        }
    }
    content.push('\n');

    // Tasks by layer
    for layer in &sorted_layers {
        let layer_num = layer_number(layer);
        content.push_str(&format!(
            "## {}. {}\n\n",
            layer_num,
            layer_display_name(layer)
        ));

        if let Some(layer_tasks) = tasks_by_layer.get(layer) {
            // Sort by task number
            let mut sorted_tasks = layer_tasks.clone();
            sorted_tasks.sort_by_key(|t| t.get("number").and_then(|n| n.as_i64()).unwrap_or(0));

            for task in sorted_tasks {
                let task_num = task.get("number").and_then(|n| n.as_i64()).unwrap_or(0);
                let title = task.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
                let description = task.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let spec_ref = task.get("spec_ref").and_then(|v| v.as_str()).unwrap_or("");

                let file = task.get("file");
                let file_path = file
                    .and_then(|f| f.get("path"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                let file_action = file
                    .and_then(|f| f.get("action"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("MODIFY");

                let depends = task
                    .get("depends")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();

                let task_id = format!("{}.{}", layer_num, task_num);

                content.push_str(&format!("### Task {}: {}\n\n", task_id, title));
                content.push_str("```yaml\n");
                content.push_str(&format!("id: {}\n", task_id));
                content.push_str(&format!("action: {}\n", file_action));
                content.push_str("status: pending\n");
                content.push_str(&format!("file: {}\n", file_path));
                content.push_str(&format!("spec_ref: {}\n", spec_ref));
                if !depends.is_empty() {
                    content.push_str(&format!("depends_on: [{}]\n", depends));
                }
                content.push_str("```\n\n");

                content.push_str(&format!("{}\n\n", description));
            }
        }
    }

    // Write the file
    let tasks_path = change_dir.join("tasks.md");
    std::fs::write(&tasks_path, &content)?;

    Ok(format!(
        "Created tasks.md for change '{}' with {} tasks at {}",
        change_id,
        total_tasks,
        tasks_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory first
        let change_dir = project_root.join("agentd/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        let args = json!({
            "change_id": "test-change",
            "tasks": [
                {
                    "layer": "data",
                    "number": 1,
                    "title": "Create MCP module structure",
                    "file": {
                        "path": "src/mcp/mod.rs",
                        "action": "CREATE"
                    },
                    "spec_ref": "mcp-spec:R1",
                    "description": "Create the MCP module with server and tools submodules"
                },
                {
                    "layer": "logic",
                    "number": 1,
                    "title": "Implement MCP server",
                    "file": {
                        "path": "src/mcp/server.rs",
                        "action": "CREATE"
                    },
                    "spec_ref": "mcp-spec:R1",
                    "description": "Implement JSON-RPC 2.0 protocol handler",
                    "depends": ["1.1"]
                },
                {
                    "layer": "testing",
                    "number": 1,
                    "title": "Add unit tests",
                    "file": {
                        "path": "src/mcp/server.rs",
                        "action": "MODIFY"
                    },
                    "spec_ref": "mcp-spec:R2",
                    "description": "Add unit tests for MCP server",
                    "depends": ["2.1"]
                }
            ]
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("Created tasks.md"));

        // Verify file was created
        let tasks_path = project_root.join("agentd/changes/test-change/tasks.md");
        assert!(tasks_path.exists());

        let content = std::fs::read_to_string(&tasks_path).unwrap();
        assert!(content.contains("id: test-change"));
        assert!(content.contains("## 1. Data Layer"));
        assert!(content.contains("## 2. Logic Layer"));
        assert!(content.contains("## 4. Testing Layer"));
        assert!(content.contains("Task 1.1"));
        assert!(content.contains("Task 2.1"));
    }
}
