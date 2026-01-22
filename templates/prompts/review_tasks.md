---
task_type: review_tasks
agent: gemini
phase: plan
variables:
  - change_id
---
# Task: Review Tasks

## Change ID
{{change_id}}

## Instructions

1. **Read tasks and context**:
   - Use: `read_file` with change_id="{{change_id}}" and file="tasks"
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Use: `list_specs` to verify coverage

2. **Check quality criteria**:
   - All spec requirements are covered by tasks
   - Dependencies are correct (no circular deps)
   - Layer organization is logical (data → logic → integration → testing)
   - File paths are accurate and specific

3. **If issues found**:
   - Call `create_tasks` MCP tool with updated data
   - Output: `<review>NEEDS_REVISION</review>`

4. **If no issues**:
   - Output: `<review>PASS</review>`

## Expected Output
- Either `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`

## Tools to Use
- `read_file`, `list_specs` (required)
- `create_tasks` (if fixes needed)
