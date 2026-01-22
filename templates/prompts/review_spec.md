---
task_type: review_spec
agent: gemini
phase: plan
variables:
  - change_id
  - spec_id
---
# Task: Review Spec '{{spec_id}}'

## Change ID
{{change_id}}

## Instructions

1. **Read the spec and context**:
   - Use: `read_file` with change_id="{{change_id}}" and file="{{spec_id}}"
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"

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
