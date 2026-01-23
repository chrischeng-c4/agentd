//! get_task MCP Tool
//!
//! Returns task instructions for agents based on task type.
//! This enables agent-agnostic task delivery through MCP.
//!
//! Templates are stored in templates/prompts/*.md with frontmatter metadata.

use super::{get_optional_string, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

/// Task types for workflows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    // Plan workflow tasks - Generation (Gemini)
    CreateProposal,
    CreateSpec,
    CreateTasks,

    // Plan workflow tasks - Review (Codex)
    ReviewProposal,
    ReviewSpec,
    ReviewTasks,

    // Plan workflow tasks - Revision (Gemini)
    ReviseProposal,
    ReviseSpec,
    ReviseTasks,

    // Implement workflow tasks
    Implement,
    ReviewImpl,
    CodeReview,
    Resolve,
}

impl TaskType {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "create_proposal" => Ok(TaskType::CreateProposal),
            "create_spec" => Ok(TaskType::CreateSpec),
            "create_tasks" => Ok(TaskType::CreateTasks),
            "review_proposal" => Ok(TaskType::ReviewProposal),
            "review_spec" => Ok(TaskType::ReviewSpec),
            "review_tasks" => Ok(TaskType::ReviewTasks),
            "revise_proposal" => Ok(TaskType::ReviseProposal),
            "revise_spec" => Ok(TaskType::ReviseSpec),
            "revise_tasks" => Ok(TaskType::ReviseTasks),
            "implement" => Ok(TaskType::Implement),
            "review_impl" => Ok(TaskType::ReviewImpl),
            "code_review" => Ok(TaskType::CodeReview),
            "resolve" => Ok(TaskType::Resolve),
            _ => anyhow::bail!("Unknown task_type: {}", s),
        }
    }

    /// Get the template filename for this task type
    fn template_name(&self) -> &'static str {
        match self {
            TaskType::CreateProposal => "create_proposal.md",
            TaskType::CreateSpec => "create_spec.md",
            TaskType::CreateTasks => "create_tasks.md",
            TaskType::ReviewProposal => "review_proposal.md",
            TaskType::ReviewSpec => "review_spec.md",
            TaskType::ReviewTasks => "review_tasks.md",
            TaskType::ReviseProposal => "revise_proposal.md",
            TaskType::ReviseSpec => "revise_spec.md",
            TaskType::ReviseTasks => "revise_tasks.md",
            TaskType::Implement => "implement.md",
            TaskType::ReviewImpl => "review_impl.md",
            TaskType::CodeReview => "code_review.md",
            TaskType::Resolve => "resolve.md",
        }
    }
}

/// Get the tool definition for get_task
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "get_task".to_string(),
        description: "Get task instructions for the current workflow step. Call this first to understand your assignment.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "task_type"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID to work on"
                },
                "task_type": {
                    "type": "string",
                    "enum": [
                        "create_proposal",
                        "create_spec",
                        "create_tasks",
                        "review_proposal",
                        "review_spec",
                        "review_tasks",
                        "revise_proposal",
                        "revise_spec",
                        "revise_tasks",
                        "implement",
                        "review_impl",
                        "code_review",
                        "resolve"
                    ],
                    "description": "The type of task to perform"
                },
                "spec_id": {
                    "type": "string",
                    "description": "Spec ID (required for create_spec, review_spec, revise_spec tasks)"
                },
                "description": {
                    "type": "string",
                    "description": "User's description of the change (for create_proposal)"
                },
                "iteration": {
                    "type": "integer",
                    "description": "Current iteration number (for review_* tasks)"
                },
                "dependencies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of spec IDs this spec depends on (for create_spec task)"
                }
            }
        }),
    }
}

/// Execute the get_task tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let task_type_str = get_required_string(args, "task_type")?;
    let task_type = TaskType::from_str(&task_type_str)?;

    let spec_id = get_optional_string(args, "spec_id");
    let description = get_optional_string(args, "description");
    let iteration = args.get("iteration").and_then(|v| v.as_i64()).unwrap_or(1);
    let dependencies: Vec<String> = args
        .get("dependencies")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Build variables map
    let mut vars = HashMap::new();
    vars.insert("project_path".to_string(), project_root.display().to_string());
    vars.insert("change_id".to_string(), change_id.clone());
    vars.insert("iteration".to_string(), iteration.to_string());

    if let Some(sid) = &spec_id {
        vars.insert("spec_id".to_string(), sid.clone());
    }
    if let Some(desc) = &description {
        vars.insert("description".to_string(), desc.clone());
    }

    // Format dependencies for template
    if dependencies.is_empty() {
        vars.insert("dependencies".to_string(), "None".to_string());
    } else {
        let deps_list = dependencies
            .iter()
            .map(|d| format!("- `{}`", d))
            .collect::<Vec<_>>()
            .join("\n");
        vars.insert("dependencies".to_string(), deps_list);
    }

    // Validate required variables for specific task types
    match task_type {
        TaskType::CreateSpec | TaskType::ReviewSpec | TaskType::ReviseSpec => {
            if spec_id.is_none() {
                anyhow::bail!("spec_id is required for {} task", task_type_str);
            }
        }
        TaskType::CreateProposal => {
            if description.is_none() {
                vars.insert("description".to_string(), "No description provided".to_string());
            }
        }
        _ => {}
    }

    // Load and render template
    let template = load_template(project_root, task_type)?;
    let rendered = render_template(&template, &vars);

    Ok(rendered)
}

/// Load a template file from templates/prompts/
fn load_template(project_root: &Path, task_type: TaskType) -> Result<String> {
    let template_path = project_root
        .join("templates")
        .join("prompts")
        .join(task_type.template_name());

    if template_path.exists() {
        let content = std::fs::read_to_string(&template_path)?;
        // Strip frontmatter (between --- delimiters)
        Ok(strip_frontmatter(&content))
    } else {
        // Fallback to embedded templates if file doesn't exist
        Ok(embedded_template(task_type))
    }
}

/// Strip YAML frontmatter from template content
fn strip_frontmatter(content: &str) -> String {
    if content.starts_with("---") {
        if let Some(end_idx) = content[3..].find("---") {
            return content[end_idx + 6..].trim_start().to_string();
        }
    }
    content.to_string()
}

/// Render template by replacing {{variable}} placeholders
fn render_template(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Embedded fallback templates (used when template files don't exist)
fn embedded_template(task_type: TaskType) -> String {
    match task_type {
        TaskType::CreateProposal => r#"# Task: Create Proposal

## Change ID
{{change_id}}

## User Request
{{description}}

## Instructions
1. Analyze the codebase
2. Call `create_proposal` MCP tool with change details

## Tools to Use
- `create_proposal` (required)
"#.to_string(),

        TaskType::ReviewProposal => r#"# Task: Review Proposal

## Change ID
{{change_id}}

## Instructions
1. Read proposal with `read_file`
2. Check quality criteria
3. Output `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`
"#.to_string(),

        TaskType::CreateSpec => r#"# Task: Create Spec '{{spec_id}}'

## Change ID
{{change_id}}

## Instructions
1. Read context with `read_file` and `list_specs`
2. Call `create_spec` MCP tool
"#.to_string(),

        TaskType::ReviewSpec => r#"# Task: Review Spec '{{spec_id}}'

## Change ID
{{change_id}}

## Instructions
1. Read spec and proposal
2. Output `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`
"#.to_string(),

        TaskType::CreateTasks => r#"# Task: Create Tasks

## Change ID
{{change_id}}

## Instructions
1. Read proposal and specs
2. Call `create_tasks` MCP tool
"#.to_string(),

        TaskType::ReviewTasks => r#"# Task: Review Tasks (Iteration {{iteration}})

## Change ID
{{change_id}}

## Instructions
1. Use `read_all_requirements` to get all files
2. Review for content/logical issues
3. Call `append_review` MCP tool with verdict
"#.to_string(),

        TaskType::ReviseProposal => r#"# Task: Revise Proposal

## Change ID
{{change_id}}

## Instructions
1. Read review feedback
2. Use `create_proposal` MCP tool to fix issues
"#.to_string(),

        TaskType::ReviseSpec => r#"# Task: Revise Spec '{{spec_id}}'

## Change ID
{{change_id}}

## Instructions
1. Read review feedback
2. Use `create_spec` MCP tool to fix issues
"#.to_string(),

        TaskType::ReviseTasks => r#"# Task: Revise Tasks

## Change ID
{{change_id}}

## Instructions
1. Read review feedback
2. Use `create_tasks` MCP tool to fix issues
"#.to_string(),

        TaskType::Implement => r#"# Task: Implement Code

## Change ID
{{change_id}}

## Instructions
1. Read requirements with `read_all_requirements`
2. Implement all tasks
3. Write tests
"#.to_string(),

        TaskType::ReviewImpl => r#"# Task: Self-Review Implementation

## Change ID
{{change_id}}

## Instructions
1. Review implementation
2. Output PASS or describe issues
"#.to_string(),

        TaskType::CodeReview => r#"# Task: Code Review (Iteration {{iteration}})

## Change ID
{{change_id}}

## Instructions
1. Read requirements and changed files
2. Write REVIEW.md with verdict
"#.to_string(),

        TaskType::Resolve => r#"# Task: Fix Review Issues

## Change ID
{{change_id}}

## Instructions
1. Read REVIEW.md
2. Fix all HIGH severity issues
"#.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_strip_frontmatter() {
        let content = r#"---
task_type: test
agent: gemini
---
# Task Content

Instructions here."#;

        let stripped = strip_frontmatter(content);
        assert!(stripped.starts_with("# Task Content"));
        assert!(!stripped.contains("task_type"));
    }

    #[test]
    fn test_render_template() {
        let template = "Change: {{change_id}}, Spec: {{spec_id}}";
        let mut vars = HashMap::new();
        vars.insert("change_id".to_string(), "my-feature".to_string());
        vars.insert("spec_id".to_string(), "auth-flow".to_string());

        let rendered = render_template(template, &vars);
        assert_eq!(rendered, "Change: my-feature, Spec: auth-flow");
    }

    #[test]
    fn test_get_task_with_template_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create templates directory and file
        let templates_dir = project_root.join("templates/prompts");
        std::fs::create_dir_all(&templates_dir).unwrap();
        std::fs::write(
            templates_dir.join("create_proposal.md"),
            r#"---
task_type: create_proposal
agent: gemini
---
# Custom Template

Change: {{change_id}}
Description: {{description}}"#,
        )
        .unwrap();

        let args = json!({
            "change_id": "test-change",
            "task_type": "create_proposal",
            "description": "Add feature X"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Custom Template"));
        assert!(result.contains("Change: test-change"));
        assert!(result.contains("Description: Add feature X"));
        assert!(!result.contains("task_type:")); // Frontmatter stripped
    }

    #[test]
    fn test_get_task_fallback_embedded() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        // No templates directory - should use embedded fallback

        let args = json!({
            "change_id": "test-change",
            "task_type": "create_proposal",
            "description": "Test"
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("# Task: Create Proposal"));
    }

    #[test]
    fn test_get_task_spec_requires_spec_id() {
        let temp_dir = TempDir::new().unwrap();
        let args = json!({
            "change_id": "test",
            "task_type": "create_spec"
        });

        let result = execute(&args, temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("spec_id is required"));
    }

    #[test]
    fn test_task_type_template_names() {
        assert_eq!(TaskType::CreateProposal.template_name(), "create_proposal.md");
        assert_eq!(TaskType::ReviewProposal.template_name(), "review_proposal.md");
        assert_eq!(TaskType::ReviseProposal.template_name(), "revise_proposal.md");
        assert_eq!(TaskType::CodeReview.template_name(), "code_review.md");
    }
}
