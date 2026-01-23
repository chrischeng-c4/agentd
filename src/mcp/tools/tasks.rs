//! create_tasks MCP Tool
//!
//! Creates a validated tasks.md file with layered task structure.

use super::{get_required_array, get_required_string, ToolDefinition};
use crate::services::tasks_service::{create_tasks, CreateTasksInput, FileActionData, TaskData};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for create_tasks
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_tasks".to_string(),
        description: "Create a validated tasks.md file with layered task structure".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "tasks"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
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

/// Execute the create_tasks tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    // Extract required fields
    let change_id = get_required_string(args, "change_id")?;
    let tasks = get_required_array(args, "tasks")?;

    // Convert tasks JSON array to TaskData
    let tasks_vec: Vec<TaskData> = tasks
        .iter()
        .filter_map(|t| {
            let file = t.get("file")?;
            Some(TaskData {
                layer: t.get("layer")?.as_str()?.to_string(),
                number: t.get("number")?.as_u64()? as u32,
                title: t.get("title")?.as_str()?.to_string(),
                file: FileActionData {
                    path: file.get("path")?.as_str()?.to_string(),
                    action: file.get("action")?.as_str()?.to_string(),
                },
                spec_ref: t.get("spec_ref")?.as_str()?.to_string(),
                description: t.get("description")?.as_str()?.to_string(),
                depends: t
                    .get("depends")
                    .and_then(|d| d.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect();

    // Create input struct and call service
    let input = CreateTasksInput {
        change_id,
        tasks: tasks_vec,
    };

    create_tasks(input, project_root)
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
