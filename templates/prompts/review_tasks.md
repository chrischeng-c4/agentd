---
task_type: review_tasks
agent: codex
phase: plan
variables:
  - change_id
  - iteration
---
# Task: Review Tasks (Iteration {{iteration}})

## Change ID
{{change_id}}

## Instructions

1. **Get all context**:
   - Use: `read_all_requirements` with change_id="{{change_id}}"
   - This retrieves proposal.md, tasks.md, and all specs/*.md

2. **Review for content/logical issues**:
   - **Coverage**: Are all spec requirements covered by tasks?
   - **Dependencies**: Are task dependencies correct (no circular deps)?
   - **Ordering**: Is layer organization logical (data → logic → integration → testing)?
   - **File paths**: Are file paths accurate and specific?
   - **Consistency**: Do tasks align with specs and proposal?

3. **Submit review**:
   - Use: `append_review` MCP tool with your findings

## Review Submission

Call `append_review` with:
- `change_id`: "{{change_id}}"
- `status`: "approved" | "needs_revision" | "rejected"
- `iteration`: {{iteration}}
- `reviewer`: "codex"
- `content`: Markdown with ## Summary, ## Issues, ## Verdict, ## Next Steps

## Verdict Guidelines
- **approved**: Tasks are complete, well-ordered, and consistent with specs
- **needs_revision**: Has issues (missing coverage, incorrect dependencies, wrong paths)
- **rejected**: Fundamental problems with task breakdown

**IMPORTANT**: Focus ONLY on content/logical issues. MCP tools guarantee correct format.

## Tools to Use
- `read_all_requirements` (required)
- `append_review` (required)
