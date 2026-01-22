//! get_task MCP Tool
//!
//! Returns task instructions for agents based on task type.
//! This enables agent-agnostic task delivery through MCP.

use super::{get_optional_string, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Task types for the plan workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    // Plan workflow tasks
    CreateProposal,
    ReviewProposal,
    CreateSpec,
    ReviewSpec,
    CreateTasks,
    ReviewTasks,
    Challenge,
    Reproposal,

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
            "review_proposal" => Ok(TaskType::ReviewProposal),
            "create_spec" => Ok(TaskType::CreateSpec),
            "review_spec" => Ok(TaskType::ReviewSpec),
            "create_tasks" => Ok(TaskType::CreateTasks),
            "review_tasks" => Ok(TaskType::ReviewTasks),
            "challenge" => Ok(TaskType::Challenge),
            "reproposal" => Ok(TaskType::Reproposal),
            "implement" => Ok(TaskType::Implement),
            "review_impl" => Ok(TaskType::ReviewImpl),
            "code_review" => Ok(TaskType::CodeReview),
            "resolve" => Ok(TaskType::Resolve),
            _ => anyhow::bail!("Unknown task_type: {}", s),
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
            "required": ["change_id", "task_type"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "description": "The change ID to work on"
                },
                "task_type": {
                    "type": "string",
                    "enum": [
                        "create_proposal",
                        "review_proposal",
                        "create_spec",
                        "review_spec",
                        "create_tasks",
                        "review_tasks",
                        "challenge",
                        "reproposal",
                        "implement",
                        "review_impl",
                        "code_review",
                        "resolve"
                    ],
                    "description": "The type of task to perform"
                },
                "spec_id": {
                    "type": "string",
                    "description": "Spec ID (required for create_spec and review_spec tasks)"
                },
                "description": {
                    "type": "string",
                    "description": "User's description of the change (for create_proposal)"
                },
                "iteration": {
                    "type": "integer",
                    "description": "Current iteration number (for review tasks)"
                }
            }
        }),
    }
}

/// Execute the get_task tool
pub fn execute(args: &Value, _project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let task_type_str = get_required_string(args, "task_type")?;
    let task_type = TaskType::from_str(&task_type_str)?;

    let spec_id = get_optional_string(args, "spec_id");
    let description = get_optional_string(args, "description");
    let iteration = args.get("iteration").and_then(|v| v.as_i64()).unwrap_or(1) as u32;

    let task = match task_type {
        TaskType::CreateProposal => {
            let desc = description.unwrap_or_else(|| "No description provided".to_string());
            create_proposal_task(&change_id, &desc)
        }
        TaskType::ReviewProposal => review_proposal_task(&change_id),
        TaskType::CreateSpec => {
            let sid = spec_id.ok_or_else(|| anyhow::anyhow!("spec_id required for create_spec"))?;
            create_spec_task(&change_id, &sid)
        }
        TaskType::ReviewSpec => {
            let sid = spec_id.ok_or_else(|| anyhow::anyhow!("spec_id required for review_spec"))?;
            review_spec_task(&change_id, &sid)
        }
        TaskType::CreateTasks => create_tasks_task(&change_id),
        TaskType::ReviewTasks => review_tasks_task(&change_id),
        TaskType::Challenge => challenge_task(&change_id, iteration),
        TaskType::Reproposal => reproposal_task(&change_id),
        TaskType::Implement => implement_task(&change_id),
        TaskType::ReviewImpl => review_impl_task(&change_id),
        TaskType::CodeReview => code_review_task(&change_id, iteration),
        TaskType::Resolve => resolve_task(&change_id),
    };

    Ok(task)
}

// =============================================================================
// Plan Workflow Tasks
// =============================================================================

fn create_proposal_task(change_id: &str, description: &str) -> String {
    format!(r#"# Task: Create Proposal

## Change ID
{change_id}

## User Request
{description}

## Instructions

1. **Analyze the codebase** using your context window:
   - Read project structure, existing code, patterns
   - Understand the technical landscape
   - Identify affected areas

2. **Determine required specs**:
   - What major components/features need detailed design?
   - Use clear, descriptive IDs (e.g., `auth-flow`, `user-model`, `api-endpoints`)

3. **Call the `create_proposal` MCP tool** with:
   - `change_id`: "{change_id}"
   - `summary`: Brief 1-sentence description
   - `why`: Detailed business/technical motivation (min 50 chars)
   - `what_changes`: Array of high-level changes
   - `impact`: Object with scope, affected_files, affected_specs, affected_code, breaking_changes

## Expected Output
- proposal.md created via `create_proposal` MCP tool

## Tools to Use
- `create_proposal` (required)
- `read_file`, `list_specs` (for context)
"#)
}

fn review_proposal_task(change_id: &str) -> String {
    format!(r#"# Task: Review Proposal

## Change ID
{change_id}

## Instructions

1. **Read the proposal**:
   - Use: `read_file` with change_id="{change_id}" and file="proposal"

2. **Check quality criteria**:
   - Summary is clear and specific (not vague)
   - Why section has compelling business/technical value
   - affected_specs list is complete and well-scoped
   - Impact analysis covers all affected areas

3. **If issues found**:
   - Call `create_proposal` MCP tool with updated data
   - Output: `<review>NEEDS_REVISION</review>`

4. **If no issues**:
   - Output: `<review>PASS</review>`

## Expected Output
- Either `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`

## Tools to Use
- `read_file` (required)
- `create_proposal` (if fixes needed)
"#)
}

fn create_spec_task(change_id: &str, spec_id: &str) -> String {
    format!(r#"# Task: Create Spec '{spec_id}'

## Change ID
{change_id}

## Instructions

1. **Read context files**:
   - Use: `read_file` with change_id="{change_id}" and file="proposal"
   - Use: `list_specs` to see existing specs
   - Read existing specs to maintain consistency

2. **Design this spec**:
   - Define clear, testable requirements (R1, R2, ...)
   - Add Mermaid diagrams if helpful (use generate_mermaid_* tools)
   - Write acceptance scenarios (WHEN/THEN format, min 3)
   - Ensure consistency with proposal.md and other specs

3. **Call the `create_spec` MCP tool** with:
   - `change_id`: "{change_id}"
   - `spec_id`: "{spec_id}"
   - `title`: Human-readable title
   - `overview`: What this spec covers (min 50 chars)
   - `requirements`: Array of requirement objects
   - `scenarios`: Array of scenario objects (min 3)
   - `flow_diagram`: Optional Mermaid diagram
   - `data_model`: Optional JSON Schema

## Expected Output
- specs/{spec_id}.md created via `create_spec` MCP tool

## Tools to Use
- `read_file`, `list_specs` (for context)
- `create_spec` (required)
- `generate_mermaid_*` (optional, for diagrams)
"#)
}

fn review_spec_task(change_id: &str, spec_id: &str) -> String {
    format!(r#"# Task: Review Spec '{spec_id}'

## Change ID
{change_id}

## Instructions

1. **Read the spec and context**:
   - Use: `read_file` with change_id="{change_id}" and file="{spec_id}"
   - Use: `read_file` with change_id="{change_id}" and file="proposal"

2. **Check quality criteria**:
   - Requirements are testable and clear
   - Scenarios cover happy path, errors, edge cases (min 3)
   - Consistent with proposal.md and other specs
   - Mermaid diagrams are correct (if present)

3. **If issues found**:
   - Call `create_spec` MCP tool with updated data
   - Output: `<review>NEEDS_REVISION</review>`

4. **If no issues**:
   - Output: `<review>PASS</review>`

## Expected Output
- Either `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`

## Tools to Use
- `read_file`, `list_specs` (required)
- `create_spec` (if fixes needed)
"#)
}

fn create_tasks_task(change_id: &str) -> String {
    format!(r#"# Task: Create Tasks

## Change ID
{change_id}

## Instructions

1. **Read all context files**:
   - Use: `read_file` with change_id="{change_id}" and file="proposal"
   - Use: `list_specs` to list all specs
   - Read all specs for detailed requirements

2. **Break down into tasks by layer**:
   - **data**: Database schemas, models, data structures
   - **logic**: Business logic, algorithms, core functionality
   - **integration**: API endpoints, external integrations
   - **testing**: Unit tests, integration tests

3. **Call the `create_tasks` MCP tool** with:
   - `change_id`: "{change_id}"
   - `tasks`: Array of task objects with layer, number, title, file, spec_ref, description, depends

## Expected Output
- tasks.md created via `create_tasks` MCP tool

## Tools to Use
- `read_file`, `list_specs` (required)
- `create_tasks` (required)
"#)
}

fn review_tasks_task(change_id: &str) -> String {
    format!(r#"# Task: Review Tasks

## Change ID
{change_id}

## Instructions

1. **Read tasks and context**:
   - Use: `read_file` with change_id="{change_id}" and file="tasks"
   - Use: `read_file` with change_id="{change_id}" and file="proposal"
   - Use: `list_specs` to verify coverage

2. **Check quality criteria**:
   - All spec requirements are covered by tasks
   - Dependencies are correct (no circular deps)
   - Layer organization is logical (data → logic → integration → testing)
   - File paths are accurate and specific

3. **If issues found**:
   - Call `create_tasks` MCP tool with updated data
   - Output: `<review>NEEDS_REVISION</review>`

4. **If no issues**:
   - Output: `<review>PASS</review>`

## Expected Output
- Either `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`

## Tools to Use
- `read_file`, `list_specs` (required)
- `create_tasks` (if fixes needed)
"#)
}

fn challenge_task(change_id: &str, iteration: u32) -> String {
    format!(r#"# Task: Challenge Proposal (Iteration {iteration})

## Change ID
{change_id}

## Instructions

1. **Get all requirements**:
   - Use: `read_all_requirements` with change_id="{change_id}"
   - This retrieves proposal.md, tasks.md, and all specs/*.md

2. **Review for content/logical issues**:
   - **Completeness**: Are all requirements covered? Missing scenarios?
   - **Consistency**: Do specs align with proposal? Do tasks cover all requirements?
   - **Technical feasibility**: Is the design implementable? Any blockers?
   - **Clarity**: Are requirements specific and testable?
   - **Dependencies**: Are task dependencies correct?

3. **Submit review**:
   - Use: `append_review` MCP tool with your findings

## Review Submission

Call `append_review` with:
- `change_id`: "{change_id}"
- `status`: "approved" | "needs_revision" | "rejected"
- `iteration`: {iteration}
- `reviewer`: "codex"
- `content`: Markdown with ## Summary, ## Issues, ## Verdict, ## Next Steps

## Verdict Guidelines
- **approved**: Complete, consistent, ready for implementation
- **needs_revision**: Has logical issues (missing requirements, inconsistencies)
- **rejected**: Fundamental design problems

**IMPORTANT**: Focus ONLY on content/logical issues. MCP tools guarantee correct format.
"#)
}

fn reproposal_task(change_id: &str) -> String {
    format!(r#"# Task: Revise Proposal Based on Feedback

## Change ID
{change_id}

## Instructions

1. **Read the review feedback**:
   - Use: `read_file` with change_id="{change_id}" and file="proposal"
   - Look for review blocks with issues to address

2. **Address each issue** using MCP tools:
   - For proposal.md: Use `create_proposal` MCP tool
   - For spec files: Use `create_spec` MCP tool
   - For tasks.md: Use `create_tasks` MCP tool

## Expected Output
- Updated files via MCP tools addressing all review feedback

## Tools to Use
- `read_file`, `list_specs` (for context)
- `create_proposal`, `create_spec`, `create_tasks` (for updates)
"#)
}

// =============================================================================
// Implement Workflow Tasks
// =============================================================================

fn implement_task(change_id: &str) -> String {
    format!(r#"# Task: Implement Code

## Change ID
{change_id}

## Instructions

1. **Read requirements**:
   - Use: `read_all_requirements` with change_id="{change_id}"

2. **Implement ALL tasks in tasks.md**:
   - Follow the layer order (data → logic → integration → testing)
   - Create/modify files as specified
   - Write tests for all implemented features

3. **Code quality**:
   - Follow existing code style and patterns
   - Add proper error handling
   - Include documentation comments

## Expected Output
- All code files created/modified per tasks.md
- Tests written for all features

## Tools to Use
- `read_all_requirements` (required)
- Standard code editing tools
"#)
}

fn review_impl_task(change_id: &str) -> String {
    format!(r#"# Task: Self-Review Implementation

## Change ID
{change_id}

## Instructions

1. **Read requirements and implementation**:
   - Use: `read_all_requirements` with change_id="{change_id}"
   - Review the code you implemented

2. **Check quality**:
   - All tasks completed
   - Tests cover all scenarios
   - Code follows patterns
   - No obvious bugs

3. **Output result**:
   - If issues: describe them and fix
   - If good: output message containing "✅" or "PASS"

## Expected Output
- Self-review result
"#)
}

fn code_review_task(change_id: &str, iteration: u32) -> String {
    format!(r#"# Task: Code Review (Iteration {iteration})

## Change ID
{change_id}

## Instructions

1. **Get requirements**:
   - Use: `read_all_requirements` with change_id="{change_id}"

2. **Get implementation summary**:
   - Use: `list_changed_files` with change_id="{change_id}"

3. **Review focus**:
   - Test results (are all tests passing?)
   - Security (any vulnerabilities?)
   - Best practices (performance, error handling)
   - Requirement compliance (does code match specs?)

4. **Write REVIEW.md** with findings

## Severity Guidelines
- **HIGH**: Failing tests, security issues, missing features
- **MEDIUM**: Style issues, missing tests, minor improvements
- **LOW**: Suggestions, nice-to-haves

## Verdict Guidelines
- **APPROVED**: All tests pass, no HIGH issues
- **NEEDS_CHANGES**: Some issues exist (fixable)
- **MAJOR_ISSUES**: Critical problems

## Tools to Use
- `read_all_requirements`, `list_changed_files` (required)
"#)
}

fn resolve_task(change_id: &str) -> String {
    format!(r#"# Task: Fix Review Issues

## Change ID
{change_id}

## Instructions

1. **Read REVIEW.md** to understand issues

2. **Fix all issues**:
   - Fix all HIGH severity issues
   - Fix MEDIUM issues if feasible
   - Update IMPLEMENTATION.md with notes

3. **Ensure tests pass** after fixes

## Expected Output
- Issues fixed
- Tests passing

## Tools to Use
- Standard code editing tools
"#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_task_create_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let args = json!({
            "change_id": "my-feature",
            "task_type": "create_proposal",
            "description": "Add user authentication"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        assert!(result.contains("# Task: Create Proposal"));
        assert!(result.contains("my-feature"));
        assert!(result.contains("Add user authentication"));
        assert!(result.contains("create_proposal"));
    }

    #[test]
    fn test_get_task_create_spec() {
        let temp_dir = TempDir::new().unwrap();
        let args = json!({
            "change_id": "my-feature",
            "task_type": "create_spec",
            "spec_id": "auth-flow"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        assert!(result.contains("# Task: Create Spec 'auth-flow'"));
        assert!(result.contains("create_spec"));
    }

    #[test]
    fn test_get_task_challenge() {
        let temp_dir = TempDir::new().unwrap();
        let args = json!({
            "change_id": "my-feature",
            "task_type": "challenge",
            "iteration": 2
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        assert!(result.contains("# Task: Challenge Proposal (Iteration 2)"));
        assert!(result.contains("append_review"));
    }

    #[test]
    fn test_get_task_missing_spec_id() {
        let temp_dir = TempDir::new().unwrap();
        let args = json!({
            "change_id": "my-feature",
            "task_type": "create_spec"
        });

        let result = execute(&args, temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("spec_id required"));
    }

    #[test]
    fn test_task_type_from_str() {
        assert_eq!(TaskType::from_str("create_proposal").unwrap(), TaskType::CreateProposal);
        assert_eq!(TaskType::from_str("challenge").unwrap(), TaskType::Challenge);
        assert!(TaskType::from_str("invalid").is_err());
    }
}
