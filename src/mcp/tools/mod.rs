//! MCP Tool Registry and Implementations
//!
//! Each tool provides structured input validation and generates properly formatted
//! markdown files, eliminating format errors from free-form LLM output.

pub mod clarifications;
pub mod implementation;
pub mod knowledge;
pub mod mermaid;
pub mod proposal;
pub mod read;
pub mod spec;
pub mod tasks;
pub mod validate;

use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Registry of available MCP tools
pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

/// Tool definition for MCP protocol
#[derive(Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl ToolRegistry {
    /// Create a new tool registry with all available tools
    pub fn new() -> Self {
        Self::all_tools()
    }

    /// Create a tool registry filtered by workflow stage
    pub fn new_for_stage(stage: &str) -> Self {
        let tools = match stage {
            "plan" => Self::plan_tools(),
            "challenge" => Self::challenge_tools(),
            "implement" => Self::implement_tools(),
            "review" => Self::review_tools(),
            "archive" => Self::archive_tools(),
            _ => Self::all_tools_vec(),
        };
        Self { tools }
    }

    /// All tools (22 total)
    fn all_tools() -> Self {
        Self {
            tools: Self::all_tools_vec(),
        }
    }

    fn all_tools_vec() -> Vec<ToolDefinition> {
        let mut tools = vec![
            clarifications::definition(),
            proposal::definition(),
            proposal::append_review_definition(),
            spec::definition(),
            tasks::definition(),
            validate::definition(),
            read::definition(),
            read::list_specs_definition(),
            knowledge::read_definition(),
            knowledge::list_definition(),
            knowledge::write_definition(),
            implementation::read_all_requirements_definition(),
            implementation::read_implementation_summary_definition(),
            implementation::list_changed_files_definition(),
        ];

        // Add all Mermaid diagram tools
        tools.extend(mermaid::definitions());

        tools
    }

    /// Plan stage tools (22 tools: all core + Mermaid)
    /// Used by: Gemini for proposal generation
    fn plan_tools() -> Vec<ToolDefinition> {
        Self::all_tools_vec()
    }

    /// Challenge stage tools (5 tools)
    /// Used by: Codex for challenging proposals
    fn challenge_tools() -> Vec<ToolDefinition> {
        vec![
            read::definition(),
            read::list_specs_definition(),
            knowledge::read_definition(),
            knowledge::list_definition(),
            validate::definition(),
        ]
    }

    /// Implement stage tools (4 tools)
    /// Used by: Claude for code implementation
    fn implement_tools() -> Vec<ToolDefinition> {
        vec![
            implementation::read_all_requirements_definition(),
            implementation::read_implementation_summary_definition(),
            implementation::list_changed_files_definition(),
            read::definition(),
        ]
    }

    /// Review stage tools (3 tools)
    /// Used by: Codex for code review
    fn review_tools() -> Vec<ToolDefinition> {
        vec![
            validate::definition(),
            proposal::append_review_definition(),
            read::definition(),
        ]
    }

    /// Archive stage tools (6 tools)
    /// Used by: Gemini for merging specs to knowledge base
    fn archive_tools() -> Vec<ToolDefinition> {
        vec![
            knowledge::read_definition(),
            knowledge::list_definition(),
            knowledge::write_definition(),
            read::definition(),
            read::list_specs_definition(),
            spec::definition(),
        ]
    }

    /// List all available tools in MCP format
    pub fn list_tools(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect()
    }

    /// Call a tool by name with the given arguments
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: &Value,
        project_root: &Path,
    ) -> Result<String> {
        match name {
            "create_clarifications" => clarifications::execute(arguments, project_root),
            "create_proposal" => proposal::execute(arguments, project_root),
            "append_review" => proposal::execute_append_review(arguments, project_root),
            "create_spec" => spec::execute(arguments, project_root),
            "create_tasks" => tasks::execute(arguments, project_root),
            "validate_change" => validate::execute(arguments, project_root).await,
            "read_file" => read::execute(arguments, project_root),
            "list_specs" => read::execute_list_specs(arguments, project_root),
            "read_knowledge" => knowledge::execute_read(arguments, project_root),
            "list_knowledge" => knowledge::execute_list(arguments, project_root),
            "write_knowledge" => knowledge::execute_write(arguments, project_root),
            "read_all_requirements" => {
                implementation::execute_read_all_requirements(arguments, project_root)
            }
            "read_implementation_summary" => {
                implementation::execute_read_implementation_summary(arguments, project_root)
            }
            "list_changed_files" => {
                implementation::execute_list_changed_files(arguments, project_root)
            }
            // Mermaid diagram tools
            name if name.starts_with("generate_mermaid_") => mermaid::call_tool(name, arguments),
            _ => anyhow::bail!("Unknown tool: {}", name),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to extract a required string field from JSON
pub fn get_required_string(args: &Value, field: &str) -> Result<String> {
    args.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing required field: {}", field))
}

/// Helper to extract an optional string field from JSON
pub fn get_optional_string(args: &Value, field: &str) -> Option<String> {
    args.get(field).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Helper to extract a required array field from JSON
pub fn get_required_array(args: &Value, field: &str) -> Result<Vec<Value>> {
    args.get(field)
        .and_then(|v| v.as_array())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing required array field: {}", field))
}

/// Helper to extract a required object field from JSON
pub fn get_required_object(args: &Value, field: &str) -> Result<Value> {
    args.get(field)
        .filter(|v| v.is_object())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Missing required object field: {}", field))
}
