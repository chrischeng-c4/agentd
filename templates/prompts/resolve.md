---
task_type: resolve
agent: claude
phase: implement
variables:
  - change_id
---
# Task: Fix Review Issues

## Change ID
{{change_id}}

## Instructions

1. **Read REVIEW.md** to understand issues

2. **Fix all issues**:
   - Fix all HIGH severity issues
   - Fix MEDIUM issues if feasible
   - Update IMPLEMENTATION.md with notes

3. **Ensure tests pass** after fixes

## Expected Output
- Issues fixed
- Tests passing

## Tools to Use
- Standard code editing tools
