---
task_type: revise_tasks
agent: gemini
phase: plan
variables:
  - change_id
---
# Task: Revise Tasks Based on Review Feedback

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Look for the latest `<review>` block with issues about tasks

2. **Read current context**:
   - Use: `read_all_requirements` with change_id="{{change_id}}"
   - Understand all specs and their requirements

3. **Address each issue** using the `create_tasks` MCP tool:
   - Fix all issues mentioned in the review
   - Ensure all spec requirements are covered
   - Fix dependency ordering if mentioned
   - Correct file paths if mentioned

4. **Verify the fix**:
   - Re-read tasks to confirm issues are resolved

## Expected Output
- Updated tasks.md via `create_tasks` MCP tool addressing all review feedback

## Tools to Use
- `read_file`, `read_all_requirements` (required)
- `create_tasks` (required)
