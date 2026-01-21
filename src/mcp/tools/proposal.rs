//! create_proposal MCP Tool
//!
//! Creates a validated proposal.md file with enforced structure.

use super::{get_optional_string, get_required_array, get_required_object, get_required_string, ToolDefinition};
use crate::models::spec_rules::SpecFormatRules;
use crate::services::proposal_service::{create_proposal, CreateProposalInput, ImpactData};
use crate::Result;
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
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let affected_code = impact
        .get("affected_code")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let breaking_changes = get_optional_string(&impact, "breaking_changes");

    // Convert what_changes array to Vec<String>
    let what_changes_vec: Vec<String> = what_changes
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    // Note: SpecFormatRules is used by the service layer indirectly
    let _prd_rules = SpecFormatRules::prd_defaults();

    // Create input struct and call service
    let input = CreateProposalInput {
        change_id,
        summary,
        why,
        what_changes: what_changes_vec,
        impact: ImpactData {
            scope,
            affected_files,
            new_files,
            affected_specs,
            affected_code,
            breaking_changes,
        },
    };

    create_proposal(input, project_root)
}

// Note: append_review function is now in the service layer (services::proposal_service::append_review)

/// Get the tool definition for append_review
pub fn append_review_definition() -> ToolDefinition {
    ToolDefinition {
        name: "append_review".to_string(),
        description: "Append a review block to an existing proposal.md file".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id", "status", "iteration", "reviewer", "content"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Unique identifier for the change (lowercase, hyphens allowed)"
                },
                "status": {
                    "type": "string",
                    "enum": ["approved", "needs_revision", "rejected"],
                    "description": "Review verdict: approved, needs_revision, or rejected"
                },
                "iteration": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Review iteration number (starts at 1)"
                },
                "reviewer": {
                    "type": "string",
                    "description": "Reviewer identifier (e.g., 'codex', 'human')"
                },
                "content": {
                    "type": "string",
                    "minLength": 50,
                    "description": "Review content in markdown format. Must include: ## Summary, ## Issues (if any), ## Verdict, ## Next Steps"
                }
            }
        }),
    }
}

/// Execute the append_review tool
pub fn execute_append_review(args: &Value, project_root: &Path) -> Result<String> {
    use crate::services::proposal_service::append_review;

    // Extract required fields
    let change_id = get_required_string(args, "change_id")?;
    let status = get_required_string(args, "status")?;
    let iteration = args
        .get("iteration")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| anyhow::anyhow!("Missing required field: iteration"))? as u32;
    let reviewer = get_required_string(args, "reviewer")?;
    let content = get_required_string(args, "content")?;

    // Validate change_id format
    if !change_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!("change_id must be lowercase alphanumeric with hyphens only");
    }

    // Validate status
    if !["approved", "needs_revision", "rejected"].contains(&status.as_str()) {
        anyhow::bail!("status must be 'approved', 'needs_revision', or 'rejected'");
    }

    // Validate content length
    if content.len() < 50 {
        anyhow::bail!("content must be at least 50 characters");
    }

    // Get proposal path
    let proposal_path = project_root
        .join("agentd/changes")
        .join(&change_id)
        .join("proposal.md");

    if !proposal_path.exists() {
        anyhow::bail!(
            "proposal.md not found for change '{}'. Run create_proposal first.",
            change_id
        );
    }

    // Append review block using service
    append_review(&proposal_path, &status, iteration, &reviewer, &content)?;

    Ok(format!(
        "Appended review block (status={}, iteration={}) to proposal.md for change '{}'",
        status, iteration, change_id
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
