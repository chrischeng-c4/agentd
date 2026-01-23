---
task_type: review_proposal
agent: codex
phase: plan
variables:
  - change_id
  - iteration
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="review_proposal")
---
# Task: Review Proposal (Iteration {{iteration}})

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Get all context**:
   - Read the proposal and clarifications

2. **Review for content/logical issues**:
   - **Clarity**: Is the summary clear and specific (not vague)?
   - **Value**: Does the Why section have compelling business/technical value?
   - **Completeness**: Is affected_specs list complete and well-scoped?
   - **Feasibility**: Is the proposed design implementable?
   - **Impact**: Does the impact analysis cover all affected areas?

3. **Submit review**:
   - Use `append_review` MCP tool with your findings

## Review Submission

Call `append_review` with:
- `status`: "approved" | "needs_revision" | "rejected"
- `iteration`: {{iteration}}
- `reviewer`: "codex"
- `content`: Markdown with ## Summary, ## Issues, ## Verdict, ## Next Steps

## Verdict Guidelines
- **approved**: Proposal is clear, complete, and ready for spec creation
- **needs_revision**: Has issues that need fixing (unclear scope, missing details)
- **rejected**: Fundamental problems with the proposal

**IMPORTANT**: Focus ONLY on content/logical issues. MCP tools guarantee correct format.

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="proposal")
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="clarifications")
```

### Generate Artifact
```
mcp__agentd-mcp__append_review(project_path="{{project_path}}", change_id="{{change_id}}", status="approved|needs_revision|rejected", iteration={{iteration}}, reviewer="codex", content="## Summary\n...\n## Issues\n...\n## Verdict\n...\n## Next Steps\n...")
```
