---
task_type: revise_spec
agent: gemini
phase: plan
variables:
  - change_id
  - spec_id
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="revise_spec", spec_id="{{spec_id}}")
---
# Task: Revise Spec '{{spec_id}}' Based on Review Feedback

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Look for the latest review block with issues about spec '{{spec_id}}'

2. **Read current spec and dependencies**:
   - Read current spec and related specs for consistency

3. **Address each issue** using the `create_spec` MCP tool:
   - Fix all issues mentioned in the review for this spec
   - Ensure requirements are testable and clear
   - Ensure scenarios cover all cases
   - Maintain consistency with proposal and dependent specs

4. **Verify the fix**:
   - Re-read the spec to confirm issues are resolved

## Expected Output
- Updated specs/{{spec_id}}.md via `create_spec` MCP tool addressing review feedback

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="proposal")
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="{{spec_id}}")
mcp__agentd-mcp__list_specs(project_path="{{project_path}}", change_id="{{change_id}}", spec_id="{{spec_id}}")
```

### Generate Artifact
```
mcp__agentd-mcp__create_spec(project_path="{{project_path}}", change_id="{{change_id}}", spec_id="{{spec_id}}", title="...", overview="...", requirements=[...], scenarios=[...])
```
