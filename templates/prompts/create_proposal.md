---
task_type: create_proposal
agent: gemini
phase: plan
variables:
  - change_id
  - description
---
# Task: Create Proposal

## Change ID
{{change_id}}

## User Request
{{description}}

## Instructions

1. **Analyze the codebase** using your context window:
   - Read project structure, existing code, patterns
   - Understand the technical landscape
   - Identify affected areas

2. **Determine required specs**:
   - What major components/features need detailed design?
   - Use clear, descriptive IDs (e.g., `auth-flow`, `user-model`, `api-endpoints`)

3. **Call the `create_proposal` MCP tool** with:
   - `change_id`: "{{change_id}}"
   - `summary`: Brief 1-sentence description
   - `why`: Detailed business/technical motivation (min 50 chars)
   - `what_changes`: Array of high-level changes
   - `impact`: Object with scope, affected_files, affected_specs, affected_code, breaking_changes

## Expected Output
- proposal.md created via `create_proposal` MCP tool

## Tools to Use
- `create_proposal` (required)
- `read_file`, `list_specs` (for context)
