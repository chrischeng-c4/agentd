//! validate_change MCP Tool
//!
//! Validates all proposal files for a change using the existing validator.

use super::{get_required_string, ToolDefinition};
use crate::cli::validate_proposal::{validate_proposal, ValidationSummary};
use crate::models::ValidationOptions;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for validate_change
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "validate_change".to_string(),
        description: "Validate all proposal files for a change".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID to validate"
                },
                "strict": {
                    "type": "boolean",
                    "default": false,
                    "description": "Treat warnings as errors"
                }
            }
        }),
    }
}

/// Execute the validate_change tool
pub async fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let strict = args
        .get("strict")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(&change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    // Run validation with JSON output mode to suppress console output
    let options = ValidationOptions::new()
        .with_strict(strict)
        .with_verbose(false)
        .with_json(true)
        .with_fix(false);

    let summary = validate_proposal(&change_id, &project_root.to_path_buf(), &options)?;

    // Format result
    format_validation_result(&change_id, &summary, strict)
}

/// Format validation result for MCP response
fn format_validation_result(
    change_id: &str,
    summary: &ValidationSummary,
    strict: bool,
) -> Result<String> {
    let passed = if strict {
        summary.is_valid_strict()
    } else {
        summary.is_valid()
    };

    let mut result = String::new();

    if passed {
        result.push_str(&format!(
            "✅ Validation PASSED for change '{}'\n\n",
            change_id
        ));
    } else {
        result.push_str(&format!(
            "❌ Validation FAILED for change '{}'\n\n",
            change_id
        ));
    }

    // Summary counts
    result.push_str("## Summary\n\n");
    result.push_str(&format!("- HIGH: {}\n", summary.high_count));
    result.push_str(&format!("- MEDIUM: {}\n", summary.medium_count));
    result.push_str(&format!("- LOW: {}\n\n", summary.low_count));

    // Errors if any
    if !summary.errors.is_empty() {
        result.push_str("## Errors\n\n");
        for error in &summary.errors {
            result.push_str(&format!("- {}\n", error));
        }
        result.push('\n');
    }

    // Stale files if any
    if !summary.stale_files.is_empty() {
        result.push_str("## Stale Files\n\n");
        for file in &summary.stale_files {
            result.push_str(&format!("- {} (modified since last validation)\n", file));
        }
        result.push('\n');
    }

    // Next steps
    if passed {
        result.push_str("## Next Steps\n\n");
        result.push_str(&format!("Run: `agentd challenge {}`\n", change_id));
    } else {
        result.push_str("## Fix Instructions\n\n");
        result.push_str("Fix the errors above and run validation again.\n");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validate_nonexistent_change() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create agentd directory structure without change
        std::fs::create_dir_all(project_root.join("agentd/changes")).unwrap();

        let args = json!({
            "change_id": "nonexistent"
        });

        let result = execute(&args, project_root).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
