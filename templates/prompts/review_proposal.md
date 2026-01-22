---
task_type: review_proposal
agent: gemini
phase: plan
variables:
  - change_id
---
# Task: Review Proposal

## Change ID
{{change_id}}

## Instructions

1. **Read the proposal**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"

2. **Check quality criteria**:
   - Summary is clear and specific (not vague)
   - Why section has compelling business/technical value
   - affected_specs list is complete and well-scoped
   - Impact analysis covers all affected areas

3. **If issues found**:
   - Call `create_proposal` MCP tool with updated data
   - Output: `<review>NEEDS_REVISION</review>`

4. **If no issues**:
   - Output: `<review>PASS</review>`

## Expected Output
- Either `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`

## Tools to Use
- `read_file` (required)
- `create_proposal` (if fixes needed)
