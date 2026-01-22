---
task_type: review_impl
agent: claude
phase: implement
variables:
  - change_id
---
# Task: Self-Review Implementation

## Change ID
{{change_id}}

## Instructions

1. **Read requirements and implementation**:
   - Use: `read_all_requirements` with change_id="{{change_id}}"
   - Review the code you implemented

2. **Check quality**:
   - All tasks completed
   - Tests cover all scenarios
   - Code follows patterns
   - No obvious bugs

3. **Output result**:
   - If issues: describe them and fix
   - If good: output message containing "PASS"

## Expected Output
- Self-review result
