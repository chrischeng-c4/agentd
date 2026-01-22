---
task_type: review_spec
agent: codex
phase: plan
variables:
  - change_id
  - spec_id
  - iteration
---
# Task: Review Spec '{{spec_id}}' (Iteration {{iteration}})

## Change ID
{{change_id}}

## Instructions

1. **Get all context**:
   - Use: `read_file` with change_id="{{change_id}}" and file="{{spec_id}}"
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Use: `list_specs` to see other specs for consistency check

2. **Review for content/logical issues**:
   - **Requirements**: Are requirements testable, specific, and complete?
   - **Scenarios**: Do scenarios cover happy path, errors, edge cases (min 3)?
   - **Consistency**: Does spec align with proposal and other specs?
   - **Dependencies**: If this spec depends on others, are interfaces consistent?
   - **Diagrams**: Are Mermaid diagrams correct (if present)?

3. **Submit review**:
   - Use: `append_review` MCP tool with your findings

## Review Submission

Call `append_review` with:
- `change_id`: "{{change_id}}"
- `status`: "approved" | "needs_revision" | "rejected"
- `iteration`: {{iteration}}
- `reviewer`: "codex"
- `content`: Markdown with ## Summary, ## Issues (referencing spec_id), ## Verdict, ## Next Steps

## Verdict Guidelines
- **approved**: Spec is complete, testable, and consistent with proposal
- **needs_revision**: Has issues (unclear requirements, missing scenarios, inconsistencies)
- **rejected**: Fundamental design problems

**IMPORTANT**: Focus ONLY on content/logical issues. MCP tools guarantee correct format.

## Tools to Use
- `read_file`, `list_specs` (required)
- `append_review` (required)
