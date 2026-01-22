//! Tasks service - Business logic for tasks creation

use crate::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;

/// File action data
#[derive(Debug, Clone)]
pub struct FileActionData {
    pub path: String,
    pub action: String, // CREATE, MODIFY, DELETE
}

/// Task data
#[derive(Debug, Clone)]
pub struct TaskData {
    pub layer: String,
    pub number: u32,
    pub title: String,
    pub file: FileActionData,
    pub spec_ref: String,
    pub description: String,
    pub depends: Vec<String>,
}

/// Input structure for creating tasks
#[derive(Debug, Clone)]
pub struct CreateTasksInput {
    pub change_id: String,
    pub tasks: Vec<TaskData>,
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

/// Create tasks file with validation
pub fn create_tasks(input: CreateTasksInput, project_root: &Path) -> Result<String> {
    // Validate tasks
    if input.tasks.is_empty() {
        anyhow::bail!("At least one task is required");
    }

    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(&input.change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Create proposal first.",
            input.change_id
        );
    }

    // Group tasks by layer
    let mut tasks_by_layer: HashMap<String, Vec<&TaskData>> = HashMap::new();
    for task in &input.tasks {
        tasks_by_layer
            .entry(task.layer.clone())
            .or_default()
            .push(task);
    }

    // Sort layers by layer number
    let mut sorted_layers: Vec<_> = tasks_by_layer.keys().cloned().collect();
    sorted_layers.sort_by_key(|l| layer_number(l));

    // Calculate summary stats
    let total_tasks = input.tasks.len();
    let mut layer_stats: HashMap<String, (u32, u32)> = HashMap::new(); // (task_count, est_files)
    for (layer, layer_tasks) in &tasks_by_layer {
        let task_count = layer_tasks.len() as u32;
        let est_files = layer_tasks
            .iter()
            .filter(|t| t.file.action == "CREATE")
            .count() as u32;
        layer_stats.insert(layer.clone(), (task_count, est_files));
    }

    // Generate tasks.md content
    let now = Utc::now();
    let mut content = String::new();

    // Frontmatter (emit both id and change_id for compatibility)
    content.push_str("---\n");
    content.push_str(&format!("id: {}\n", input.change_id));
    content.push_str(&format!("change_id: {}\n", input.change_id));
    content.push_str("type: tasks\n");
    content.push_str("version: 1\n");
    content.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("proposal_ref: {}\n", input.change_id));

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

    // Wrap tasks content in XML
    content.push_str("<tasks>\n\n");

    // Title
    content.push_str("# Implementation Tasks\n\n");

    // Overview
    content.push_str("## Overview\n\n");
    content.push_str(&format!(
        "This document outlines {} implementation tasks for change `{}`.\n\n",
        total_tasks, input.change_id
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
            sorted_tasks.sort_by_key(|t| t.number);

            for task in sorted_tasks {
                let task_id = format!("{}.{}", layer_num, task.number);

                content.push_str(&format!("### Task {}: {}\n\n", task_id, task.title));
                content.push_str("```yaml\n");
                content.push_str(&format!("id: {}\n", task_id));
                content.push_str(&format!("action: {}\n", task.file.action));
                content.push_str("status: pending\n");
                content.push_str(&format!("file: {}\n", task.file.path));
                content.push_str(&format!("spec_ref: {}\n", task.spec_ref));
                if !task.depends.is_empty() {
                    content.push_str(&format!("depends_on: [{}]\n", task.depends.join(", ")));
                }
                content.push_str("```\n\n");

                content.push_str(&format!("{}\n\n", task.description));
            }
        }
    }

    // Close tasks XML tag
    content.push_str("</tasks>\n");

    // Write the file
    let tasks_path = change_dir.join("tasks.md");
    std::fs::write(&tasks_path, &content)?;

    Ok(format!(
        "Created tasks.md for change '{}' with {} tasks at {}",
        input.change_id,
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

        let input = CreateTasksInput {
            change_id: "test-change".to_string(),
            tasks: vec![
                TaskData {
                    layer: "data".to_string(),
                    number: 1,
                    title: "Create MCP module structure".to_string(),
                    file: FileActionData {
                        path: "src/mcp/mod.rs".to_string(),
                        action: "CREATE".to_string(),
                    },
                    spec_ref: "mcp-spec:R1".to_string(),
                    description: "Create the MCP module with server and tools submodules"
                        .to_string(),
                    depends: vec![],
                },
                TaskData {
                    layer: "logic".to_string(),
                    number: 1,
                    title: "Implement MCP server".to_string(),
                    file: FileActionData {
                        path: "src/mcp/server.rs".to_string(),
                        action: "CREATE".to_string(),
                    },
                    spec_ref: "mcp-spec:R1".to_string(),
                    description: "Implement JSON-RPC 2.0 protocol handler".to_string(),
                    depends: vec!["1.1".to_string()],
                },
                TaskData {
                    layer: "testing".to_string(),
                    number: 1,
                    title: "Add unit tests".to_string(),
                    file: FileActionData {
                        path: "src/mcp/server.rs".to_string(),
                        action: "MODIFY".to_string(),
                    },
                    spec_ref: "mcp-spec:R2".to_string(),
                    description: "Add unit tests for MCP server".to_string(),
                    depends: vec!["2.1".to_string()],
                },
            ],
        };

        let result = create_tasks(input, project_root).unwrap();
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

    #[test]
    fn test_create_tasks_empty() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let input = CreateTasksInput {
            change_id: "test".to_string(),
            tasks: vec![],
        };

        let result = create_tasks(input, project_root);
        assert!(result.is_err());
    }
}
