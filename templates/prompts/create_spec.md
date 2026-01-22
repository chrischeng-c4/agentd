---
task_type: create_spec
agent: gemini
phase: plan
variables:
  - change_id
  - spec_id
  - dependencies
---
# Task: Create Spec '{{spec_id}}'

## Change ID
{{change_id}}

## Dependencies
{{dependencies}}

## Instructions

1. **Read dependent specs first** (if any dependencies listed above):
   - For each dependency spec, use: `read_file` with change_id="{{change_id}}" and file="<dependency-spec-id>"
   - Understand the interfaces, data models, and requirements of dependencies
   - Your spec must be consistent with these dependencies

2. **Read context files**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Use: `list_specs` to see existing specs
   - Read existing specs to maintain consistency

3. **Design this spec**:
   - Define clear, testable requirements (R1, R2, ...)
   - Add Mermaid diagrams if helpful (use generate_mermaid_* tools)
   - Write acceptance scenarios (WHEN/THEN format, min 3)
   - Ensure consistency with proposal.md, dependencies, and other specs

4. **Call the `create_spec` MCP tool** with:
   - `change_id`: "{{change_id}}"
   - `spec_id`: "{{spec_id}}"
   - `title`: Human-readable title
   - `overview`: What this spec covers (min 50 chars)
   - `requirements`: Array of requirement objects
   - `scenarios`: Array of scenario objects (min 3)
   - `flow_diagram`: Optional Mermaid diagram
   - `data_model`: Optional JSON Schema

## Expected Output
- specs/{{spec_id}}.md created via `create_spec` MCP tool

## Tools to Use
- `read_file`, `list_specs` (for context and dependencies)
- `create_spec` (required)
- `generate_mermaid_*` (optional, for diagrams)
