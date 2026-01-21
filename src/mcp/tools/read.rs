//! read_file MCP Tool
//!
//! Reads files from a change directory to support progressive workflows.

use super::{get_optional_string, get_required_string, ToolDefinition};
use crate::services::file_service::{list_specs, read_file};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for read_file
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_file".to_string(),
        description: "Read a file from a change directory (proposal.md, specs/*.md, tasks.md)"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID to read from"
                },
                "file": {
                    "type": "string",
                    "description": "File to read: 'proposal', 'tasks', or spec name like 'my-spec'. Defaults to 'proposal'",
                    "default": "proposal"
                }
            }
        }),
    }
}

/// Execute the read_file tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let file = get_optional_string(args, "file").unwrap_or_else(|| "proposal".to_string());
    read_file(&change_id, &file, project_root)
}

/// Get the tool definition for list_specs
pub fn list_specs_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_specs".to_string(),
        description: "List all spec files in a change directory".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID to list specs for"
                }
            }
        }),
    }
}

/// Execute the list_specs tool
pub fn execute_list_specs(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    list_specs(&change_id, project_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with proposal
        let change_dir = project_root.join("agentd/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nThis is a test.",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Test Proposal"));
        assert!(result.contains("This is a test"));
    }

    #[test]
    fn test_read_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with spec
        let specs_dir = project_root.join("agentd/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("my-feature.md"),
            "# My Feature Spec\n\nRequirements here.",
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "file": "my-feature"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# My Feature Spec"));
    }

    #[test]
    fn test_list_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with specs
        let specs_dir = project_root.join("agentd/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("spec-a.md"), "# Spec A").unwrap();
        std::fs::write(specs_dir.join("spec-b.md"), "# Spec B").unwrap();
        std::fs::write(specs_dir.join("_skeleton.md"), "# Skeleton").unwrap();

        let args = json!({
            "change_id": "test-change"
        });

        let result = execute_list_specs(&args, project_root).unwrap();
        assert!(result.contains("spec-a"));
        assert!(result.contains("spec-b"));
        assert!(!result.contains("_skeleton"));
        assert!(result.contains("Total: 2 spec(s)"));
    }
}
