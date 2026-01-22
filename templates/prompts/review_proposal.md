---
task_type: review_proposal
agent: codex
phase: plan
variables:
  - change_id
  - iteration
---
# Task: Review Proposal (Iteration {{iteration}})

## Change ID
{{change_id}}

## Instructions

1. **Get all context**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"

2. **Review for content/logical issues**:
   - **Clarity**: Is the summary clear and specific (not vague)?
   - **Value**: Does the Why section have compelling business/technical value?
   - **Completeness**: Is affected_specs list complete and well-scoped?
   - **Feasibility**: Is the proposed design implementable?
   - **Impact**: Does the impact analysis cover all affected areas?

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
- **approved**: Proposal is clear, complete, and ready for spec creation
- **needs_revision**: Has issues that need fixing (unclear scope, missing details)
- **rejected**: Fundamental problems with the proposal

**IMPORTANT**: Focus ONLY on content/logical issues. MCP tools guarantee correct format.

## Tools to Use
- `read_file` (required)
- `append_review` (required)
