//! read_file MCP Tool
//!
//! Reads files from a change directory to support progressive workflows.

use super::{get_optional_string, get_required_string, ToolDefinition};
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

    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(&change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    // Determine which file to read
    let file_path = match file.as_str() {
        "proposal" => change_dir.join("proposal.md"),
        "tasks" => change_dir.join("tasks.md"),
        spec_name => {
            // Try as a spec file
            let spec_path = change_dir.join("specs").join(format!("{}.md", spec_name));
            if spec_path.exists() {
                spec_path
            } else {
                // Maybe they included .md already
                let spec_path_with_ext = change_dir.join("specs").join(spec_name);
                if spec_path_with_ext.exists() {
                    spec_path_with_ext
                } else {
                    anyhow::bail!(
                        "File not found: '{}'. Use 'proposal', 'tasks', or a spec name.",
                        file
                    );
                }
            }
        }
    };

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let content = std::fs::read_to_string(&file_path)?;

    Ok(format!(
        "# File: {}\n\n{}",
        file_path.strip_prefix(&change_dir).unwrap_or(&file_path).display(),
        content
    ))
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

    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(&change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    let specs_dir = change_dir.join("specs");
    if !specs_dir.exists() {
        return Ok("No specs directory found.".to_string());
    }

    let mut specs = Vec::new();
    for entry in std::fs::read_dir(&specs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            if let Some(name) = path.file_stem() {
                let name_str = name.to_string_lossy();
                // Skip skeleton files
                if !name_str.starts_with('_') {
                    specs.push(name_str.to_string());
                }
            }
        }
    }

    if specs.is_empty() {
        return Ok("No spec files found.".to_string());
    }

    specs.sort();

    let mut result = format!("# Specs for change '{}'\n\n", change_id);
    for spec in &specs {
        result.push_str(&format!("- {}\n", spec));
    }
    result.push_str(&format!("\nTotal: {} spec(s)", specs.len()));

    Ok(result)
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
