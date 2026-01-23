---
task_type: revise_tasks
agent: gemini
phase: plan
variables:
  - change_id
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="revise_tasks")
---
# Task: Revise Tasks Based on Review Feedback

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Look for the latest review block with issues about tasks

2. **Read current context**:
   - Understand all specs and their requirements

3. **Address each issue** using the `create_tasks` MCP tool:
   - Fix all issues mentioned in the review
   - Ensure all spec requirements are covered
   - Fix dependency ordering if mentioned
   - Correct file paths if mentioned

4. **Verify the fix**:
   - Re-read tasks to confirm issues are resolved

## Expected Output
- Updated tasks.md via `create_tasks` MCP tool addressing all review feedback

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="proposal")
mcp__agentd-mcp__read_all_requirements(project_path="{{project_path}}", change_id="{{change_id}}")
```

### Generate Artifact
```
mcp__agentd-mcp__create_tasks(project_path="{{project_path}}", change_id="{{change_id}}", tasks=[{layer:"data", number:1, title:"...", file:{path:"...", action:"CREATE"}, spec_ref:"...", description:"...", depends:[]}])
```
