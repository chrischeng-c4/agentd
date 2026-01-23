---
task_type: review_spec
agent: codex
phase: plan
variables:
  - change_id
  - spec_id
  - iteration
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="review_spec", spec_id="{{spec_id}}")
---
# Task: Review Spec '{{spec_id}}' (Iteration {{iteration}})

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Get all context**:
   - Read the spec, proposal, and dependency specs

2. **Review for content/logical issues**:
   - **Requirements**: Are requirements testable, specific, and complete?
   - **Scenarios**: Do scenarios cover happy path, errors, edge cases (min 3)?
   - **Consistency**: Does spec align with proposal and other specs?
   - **Dependencies**: If this spec depends on others, are interfaces consistent?
   - **Diagrams**: Are Mermaid diagrams correct (if present)?

3. **Submit review**:
   - Use `append_review` MCP tool with your findings

## Review Submission

Call `append_review` with:
- `status`: "approved" | "needs_revision" | "rejected"
- `iteration`: {{iteration}}
- `reviewer`: "codex"
- `content`: Markdown with ## Summary, ## Issues (referencing spec_id), ## Verdict, ## Next Steps

## Verdict Guidelines
- **approved**: Spec is complete, testable, and consistent with proposal
- **needs_revision**: Has issues (unclear requirements, missing scenarios, inconsistencies)
- **rejected**: Fundamental design problems

**IMPORTANT**: Focus ONLY on content/logical issues. MCP tools guarantee correct format.

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="{{spec_id}}")
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="proposal")
mcp__agentd-mcp__list_specs(project_path="{{project_path}}", change_id="{{change_id}}", spec_id="{{spec_id}}")
# Then read each dependency spec returned by list_specs
```

### Generate Artifact
```
mcp__agentd-mcp__append_review(project_path="{{project_path}}", change_id="{{change_id}}", status="approved|needs_revision|rejected", iteration={{iteration}}, reviewer="codex", content="## Summary\n...\n## Issues\n...\n## Verdict\n...\n## Next Steps\n...")
```
