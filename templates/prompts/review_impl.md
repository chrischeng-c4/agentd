---
task_type: review_impl
agent: claude
phase: implement
variables:
  - change_id
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="review_impl")
---
# Task: Self-Review Implementation

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Read requirements and implementation**:
   - Review the code you implemented against requirements

2. **Check quality**:
   - All tasks completed
   - Tests cover all scenarios
   - Code follows patterns
   - No obvious bugs

3. **Output result**:
   - If issues: describe them and fix
   - If good: output message containing "PASS"

## Expected Output
- Self-review result

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_all_requirements(project_path="{{project_path}}", change_id="{{change_id}}")
mcp__agentd-mcp__list_changed_files(project_path="{{project_path}}", change_id="{{change_id}}")
```

### Generate Artifact
Output "PASS" if implementation is complete, or describe issues to fix.
