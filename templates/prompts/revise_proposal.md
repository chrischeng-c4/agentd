---
task_type: revise_proposal
agent: gemini
phase: plan
variables:
  - change_id
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="revise_proposal")
---
# Task: Revise Proposal Based on Review Feedback

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Look for the latest review block with issues to address

2. **Address each issue** using the `create_proposal` MCP tool:
   - Fix all issues mentioned in the review
   - Ensure the revised proposal is complete and clear
   - Do NOT modify specs or tasks at this stage

3. **Verify the fix**:
   - Re-read the proposal to confirm issues are resolved

## Expected Output
- Updated proposal.md via `create_proposal` MCP tool addressing all review feedback

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="proposal")
mcp__agentd-mcp__read_file(project_path="{{project_path}}", change_id="{{change_id}}", file="clarifications")
```

### Generate Artifact
```
mcp__agentd-mcp__create_proposal(project_path="{{project_path}}", change_id="{{change_id}}", summary="...", why="...", what_changes=["..."], impact={...})
```
