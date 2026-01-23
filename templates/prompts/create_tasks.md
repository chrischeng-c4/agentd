---
task_type: create_tasks
agent: gemini
phase: plan
variables:
  - change_id
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="create_tasks")
---
# Task: Create Tasks

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Read all context files**:
   - Read proposal and all specs for detailed requirements

2. **Break down into tasks by layer**:
   - **data**: Database schemas, models, data structures
   - **logic**: Business logic, algorithms, core functionality
   - **integration**: API endpoints, external integrations
   - **testing**: Unit tests, integration tests

3. **Call the `create_tasks` MCP tool** with:
   - `tasks`: Array of task objects with layer, number, title, file, spec_ref, description, depends

## Expected Output
- tasks.md created via `create_tasks` MCP tool

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="proposal")
mcp__agentd-mcp__list_specs(project_path="{{project_path}}", change_id="{{change_id}}")
# Then read each spec returned by list_specs
```

### Generate Artifact
```
mcp__agentd-mcp__create_tasks(project_path="{{project_path}}", change_id="{{change_id}}", tasks=[{layer:"data", number:1, title:"...", file:{path:"...", action:"CREATE"}, spec_ref:"...", description:"...", depends:[]}])
```
