---
task_type: revise_proposal
agent: gemini
phase: plan
variables:
  - change_id
---
# Task: Revise Proposal Based on Review Feedback

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Look for the latest `<review>` block with issues to address

2. **Address each issue** using the `create_proposal` MCP tool:
   - Fix all issues mentioned in the review
   - Ensure the revised proposal is complete and clear
   - Do NOT modify specs or tasks at this stage

3. **Verify the fix**:
   - Re-read the proposal to confirm issues are resolved

## Expected Output
- Updated proposal.md via `create_proposal` MCP tool addressing all review feedback

## Tools to Use
- `read_file` (required)
- `create_proposal` (required)
