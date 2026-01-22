---
task_type: implement
agent: claude
phase: implement
variables:
  - change_id
---
# Task: Implement Code

## Change ID
{{change_id}}

## Instructions

1. **Read requirements**:
   - Use: `read_all_requirements` with change_id="{{change_id}}"

2. **Implement ALL tasks in tasks.md**:
   - Follow the layer order (data → logic → integration → testing)
   - Create/modify files as specified
   - Write tests for all implemented features

3. **Code quality**:
   - Follow existing code style and patterns
   - Add proper error handling
   - Include documentation comments

## Expected Output
- All code files created/modified per tasks.md
- Tests written for all features

## Tools to Use
- `read_all_requirements` (required)
- Standard code editing tools
