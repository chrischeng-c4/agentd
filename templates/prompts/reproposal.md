---
task_type: reproposal
agent: gemini
phase: plan
variables:
  - change_id
---
# Task: Revise Proposal Based on Feedback

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Use: `read_file` with change_id="{{change_id}}" and file="proposal"
   - Look for review blocks with issues to address

2. **Address each issue** using MCP tools:
   - For proposal.md: Use `create_proposal` MCP tool
   - For spec files: Use `create_spec` MCP tool
   - For tasks.md: Use `create_tasks` MCP tool

## Expected Output
- Updated files via MCP tools addressing all review feedback

## Tools to Use
- `read_file`, `list_specs` (for context)
- `create_proposal`, `create_spec`, `create_tasks` (for updates)
