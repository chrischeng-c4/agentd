---
task_type: resolve
agent: claude
phase: implement
variables:
  - change_id
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="resolve")
---
# Task: Fix Review Issues

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

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

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="review")
mcp__agentd-mcp__read_all_requirements(project_path="{{project_path}}", change_id="{{change_id}}")
```

### Generate Artifact
Use standard code editing tools (Read, Edit, Write) to fix the issues.
