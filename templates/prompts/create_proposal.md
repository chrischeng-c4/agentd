---
task_type: create_proposal
agent: gemini
phase: plan
variables:
  - change_id
  - description
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="create_proposal")
---
# Task: Create Proposal

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## User Request
{{description}}

## Instructions

1. **Analyze the codebase** using your context window:
   - Read project structure, existing code, patterns
   - Understand the technical landscape
   - Identify affected areas

2. **Determine required specs**:
   - What major components/features need detailed design?
   - Use clear, descriptive IDs (e.g., `auth-flow`, `user-model`, `api-endpoints`)

3. **Call the `create_proposal` MCP tool** with:
   - `change_id`: "{{change_id}}"
   - `summary`: Brief 1-sentence description
   - `why`: Detailed business/technical motivation (min 50 chars)
   - `what_changes`: Array of high-level changes
   - `impact`: Object with scope, affected_files, affected_specs, affected_code, breaking_changes

## Expected Output
- proposal.md created via `create_proposal` MCP tool

## MCP Tools

### Read Context
```
mcp__agentd-mcp__list_knowledge(project_path="{{project_path}}")
mcp__agentd-mcp__read_knowledge(project_path="{{project_path}}", path="index.md")
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="clarifications")
```

### Generate Artifact
```
mcp__agentd-mcp__create_proposal(project_path="{{project_path}}", change_id="{{change_id}}", summary="...", why="...", what_changes=["..."], impact={...})
```
