---
task_type: create_spec
agent: gemini
phase: plan
variables:
  - change_id
  - spec_id
  - dependencies
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="create_spec", spec_id="{{spec_id}}")
---
# Task: Create Spec '{{spec_id}}'

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Dependencies
{{dependencies}}

## Instructions

1. **Read dependent specs first** (if any dependencies listed above):
   - Understand the interfaces, data models, and requirements of dependencies
   - Your spec must be consistent with these dependencies

2. **Read context files**:
   - Read proposal and existing specs to maintain consistency

3. **Design this spec**:
   - Define clear, testable requirements (R1, R2, ...)
   - Add Mermaid diagrams if helpful
   - Write acceptance scenarios (WHEN/THEN format, min 3)
   - Ensure consistency with proposal.md, dependencies, and other specs

4. **Call the `create_spec` MCP tool** with:
   - `spec_id`: "{{spec_id}}"
   - `title`: Human-readable title
   - `overview`: What this spec covers (min 50 chars)
   - `requirements`: Array of requirement objects
   - `scenarios`: Array of scenario objects (min 3)
   - `flow_diagram`: Optional Mermaid diagram
   - `data_model`: Optional JSON Schema

## Expected Output
- specs/{{spec_id}}.md created via `create_spec` MCP tool

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="proposal")
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="clarifications")
mcp__agentd-mcp__list_specs(project_path="{{project_path}}", change_id="{{change_id}}", spec_id="{{spec_id}}")
# Then read each dependency spec returned by list_specs:
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="<dep-spec-id>")
```

### Generate Artifact
```
mcp__agentd-mcp__create_spec(project_path="{{project_path}}", change_id="{{change_id}}", spec_id="{{spec_id}}", title="...", overview="...", requirements=[...], scenarios=[...])
```
