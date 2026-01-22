---
task_type: revise_spec
agent: gemini
phase: plan
variables:
  - change_id
  - spec_id
---
# Task: Revise Spec '{{spec_id}}' Based on Review Feedback

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Look for the latest `<review>` block with issues about spec '{{spec_id}}'

2. **Read current spec and dependencies**:
   - Use: `read_file` with change_id="{{change_id}}" and file="{{spec_id}}"
   - Use: `list_specs` to check related specs for consistency

3. **Address each issue** using the `create_spec` MCP tool:
   - Fix all issues mentioned in the review for this spec
   - Ensure requirements are testable and clear
   - Ensure scenarios cover all cases
   - Maintain consistency with proposal and dependent specs

4. **Verify the fix**:
   - Re-read the spec to confirm issues are resolved

## Expected Output
- Updated specs/{{spec_id}}.md via `create_spec` MCP tool addressing review feedback

## Tools to Use
- `read_file`, `list_specs` (required)
- `create_spec` (required)
