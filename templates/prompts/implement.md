---
task_type: implement
agent: claude
phase: implement
variables:
  - change_id
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="implement")
---
# Task: Implement Code

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Read requirements**:
   - Get all specs and tasks for implementation

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

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_all_requirements(project_path="{{project_path}}", change_id="{{change_id}}")
```

### Generate Artifact
Use standard code editing tools (Read, Edit, Write) to implement the code.
