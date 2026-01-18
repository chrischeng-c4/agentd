//! create_proposal MCP Tool
//!
//! Creates a validated proposal.md file with enforced structure.

use super::{get_optional_string, get_required_array, get_required_object, get_required_string, ToolDefinition};
use crate::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for create_proposal
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_proposal".to_string(),
        description: "Create a validated proposal.md file with enforced structure".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id", "summary", "why", "what_changes", "impact"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Unique identifier for the change (lowercase, hyphens allowed)"
                },
                "summary": {
                    "type": "string",
                    "minLength": 10,
                    "description": "Brief one-line summary of the change"
                },
                "why": {
                    "type": "string",
                    "minLength": 50,
                    "description": "Detailed explanation of why this change is needed"
                },
                "what_changes": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "description": "List of high-level changes being made"
                },
                "impact": {
                    "type": "object",
                    "required": ["scope", "affected_files"],
                    "properties": {
                        "scope": {
                            "enum": ["patch", "minor", "major"],
                            "description": "Impact scope level"
                        },
                        "affected_files": {
                            "type": "integer",
                            "minimum": 1,
                            "description": "Estimated number of files affected"
                        },
                        "new_files": {
                            "type": "integer",
                            "default": 0,
                            "description": "Estimated number of new files"
                        },
                        "affected_specs": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of spec IDs affected"
                        },
                        "affected_code": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of code paths affected"
                        },
                        "breaking_changes": {
                            "type": ["string", "null"],
                            "description": "Description of breaking changes if any"
                        }
                    }
                }
            }
        }),
    }
}

/// Execute the create_proposal tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    // Extract required fields
    let change_id = get_required_string(args, "change_id")?;
    let summary = get_required_string(args, "summary")?;
    let why = get_required_string(args, "why")?;
    let what_changes = get_required_array(args, "what_changes")?;
    let impact = get_required_object(args, "impact")?;

    // Validate change_id format
    if !change_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!("change_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate summary length
    if summary.len() < 10 {
        anyhow::bail!("summary must be at least 10 characters");
    }

    // Validate why length
    if why.len() < 50 {
        anyhow::bail!("why must be at least 50 characters");
    }

    // Extract impact fields
    let scope = get_required_string(&impact, "scope")?;
    let affected_files = impact
        .get("affected_files")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("Missing impact.affected_files"))?;
    let new_files = impact.get("new_files").and_then(|v| v.as_i64()).unwrap_or(0);
    let affected_specs = impact
        .get("affected_specs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let affected_code = impact
        .get("affected_code")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let breaking_changes = get_optional_string(&impact, "breaking_changes");

    // Validate scope
    if !["patch", "minor", "major"].contains(&scope.as_str()) {
        anyhow::bail!("impact.scope must be 'patch', 'minor', or 'major'");
    }

    // Create change directory
    let change_dir = project_root.join("agentd/changes").join(&change_id);
    std::fs::create_dir_all(&change_dir)?;

    // Generate proposal.md content
    let now = Utc::now();
    let mut content = String::new();

    // Frontmatter
    content.push_str("---\n");
    content.push_str(&format!("id: {}\n", change_id));
    content.push_str("type: proposal\n");
    content.push_str("version: 1\n");
    content.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    content.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    content.push_str("author: mcp\n");
    content.push_str("status: proposed\n");
    content.push_str("iteration: 1\n");
    content.push_str(&format!("summary: \"{}\"\n", summary.replace('"', "\\\"")));

    // Impact section in frontmatter
    content.push_str("impact:\n");
    content.push_str(&format!("  scope: {}\n", scope));
    content.push_str(&format!("  affected_files: {}\n", affected_files));
    content.push_str(&format!("  new_files: {}\n", new_files));

    // Affected specs
    if !affected_specs.is_empty() {
        content.push_str("affected_specs:\n");
        for spec in &affected_specs {
            if let Some(spec_id) = spec.as_str() {
                content.push_str(&format!("  - id: {}\n", spec_id));
                content.push_str(&format!("    path: specs/{}.md\n", spec_id));
            }
        }
    }

    content.push_str("---\n\n");

    // Title
    content.push_str(&format!("# Change: {}\n\n", change_id));

    // Summary section
    content.push_str("## Summary\n\n");
    content.push_str(&format!("{}\n\n", summary));

    // Why section
    content.push_str("## Why\n\n");
    content.push_str(&format!("{}\n\n", why));

    // What Changes section
    content.push_str("## What Changes\n\n");
    for change in &what_changes {
        if let Some(change_text) = change.as_str() {
            content.push_str(&format!("- {}\n", change_text));
        }
    }
    content.push('\n');

    // Impact section in markdown
    content.push_str("## Impact\n\n");
    content.push_str(&format!("- **Scope**: {}\n", scope));
    content.push_str(&format!("- **Affected Files**: ~{}\n", affected_files));
    content.push_str(&format!("- **New Files**: ~{}\n", new_files));

    if !affected_specs.is_empty() {
        content.push_str("- **Affected Specs**:\n");
        for spec in &affected_specs {
            if let Some(spec_id) = spec.as_str() {
                content.push_str(&format!("  - `{}`\n", spec_id));
            }
        }
    }

    if !affected_code.is_empty() {
        content.push_str("- **Affected Code**:\n");
        for code in &affected_code {
            if let Some(code_path) = code.as_str() {
                content.push_str(&format!("  - `{}`\n", code_path));
            }
        }
    }

    if let Some(breaking) = &breaking_changes {
        content.push_str(&format!("- **Breaking Changes**: {}\n", breaking));
    }
    content.push('\n');

    // Write the file
    let proposal_path = change_dir.join("proposal.md");
    std::fs::write(&proposal_path, &content)?;

    // Create specs directory
    let specs_dir = change_dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    // Initialize STATE.yaml
    let state_content = format!(
        r#"change_id: {}
schema_version: "2.0"
created_at: {}
updated_at: {}
phase: proposed
iteration: 1
last_action: create_proposal (mcp)
checksums: {{}}
validations: []
"#,
        change_id,
        now.to_rfc3339(),
        now.to_rfc3339()
    );
    std::fs::write(change_dir.join("STATE.yaml"), state_content)?;

    Ok(format!(
        "Created proposal.md for change '{}' at {}",
        change_id,
        proposal_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create agentd directory structure
        std::fs::create_dir_all(project_root.join("agentd/changes")).unwrap();

        let args = json!({
            "change_id": "test-change",
            "summary": "This is a test change with sufficient length",
            "why": "This change is needed because we want to test the MCP tool functionality and ensure it works correctly",
            "what_changes": [
                "Add new feature X",
                "Modify existing module Y"
            ],
            "impact": {
                "scope": "minor",
                "affected_files": 5,
                "new_files": 2,
                "affected_specs": ["mcp-spec"],
                "affected_code": ["src/mcp/"]
            }
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("Created proposal.md"));

        // Verify file was created
        let proposal_path = project_root.join("agentd/changes/test-change/proposal.md");
        assert!(proposal_path.exists());

        let content = std::fs::read_to_string(&proposal_path).unwrap();
        assert!(content.contains("id: test-change"));
        assert!(content.contains("## Summary"));
        assert!(content.contains("## Why"));
        assert!(content.contains("## What Changes"));
    }
}
