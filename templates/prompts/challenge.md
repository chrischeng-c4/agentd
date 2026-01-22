---
task_type: challenge
agent: codex
phase: plan
variables:
  - change_id
  - iteration
---
# Task: Challenge Proposal (Iteration {{iteration}})

## Change ID
{{change_id}}

## Instructions

1. **Get all requirements**:
   - Use: `read_all_requirements` with change_id="{{change_id}}"
   - This retrieves proposal.md, tasks.md, and all specs/*.md

2. **Review for content/logical issues**:
   - **Completeness**: Are all requirements covered? Missing scenarios?
   - **Consistency**: Do specs align with proposal? Do tasks cover all requirements?
   - **Technical feasibility**: Is the design implementable? Any blockers?
   - **Clarity**: Are requirements specific and testable?
   - **Dependencies**: Are task dependencies correct?

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
- **approved**: Complete, consistent, ready for implementation
- **needs_revision**: Has logical issues (missing requirements, inconsistencies)
- **rejected**: Fundamental design problems

**IMPORTANT**: Focus ONLY on content/logical issues. MCP tools guarantee correct format.
