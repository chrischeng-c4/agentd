---
task_type: create_tasks
agent: gemini
phase: plan
variables:
  - change_id
---
# Task: Create Tasks

## Change ID
{{change_id}}

## Instructions

1. **Read all context files**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Use: `list_specs` to list all specs
   - Read all specs for detailed requirements

2. **Break down into tasks by layer**:
   - **data**: Database schemas, models, data structures
   - **logic**: Business logic, algorithms, core functionality
   - **integration**: API endpoints, external integrations
   - **testing**: Unit tests, integration tests

3. **Call the `create_tasks` MCP tool** with:
   - `change_id`: "{{change_id}}"
   - `tasks`: Array of task objects with layer, number, title, file, spec_ref, description, depends

## Expected Output
- tasks.md created via `create_tasks` MCP tool

## Tools to Use
- `read_file`, `list_specs` (required)
- `create_tasks` (required)
