---
name: agentd:proposal
description: Generate proposal using Agentd
user-invocable: true
---

# /agentd:proposal

Generate spec-driven proposal with PRD, Technical Design, and Tickets.

## Your Role

Your job is to:
1. **Understand** - Parse change-id and description from user input
2. **Clarify** - If unclear, ask user to confirm
3. **Transform** - Build the correct CLI command
4. **Execute** - Run the command and report results

**Important**: Do NOT explore the codebase yourself. Code exploration is done by Gemini internally when the CLI runs.

## Parsing Rules

User input format: `/agentd:proposal <change-id> "<description>"`

Examples:
- Input: `/agentd:proposal add-oauth "Add OAuth with Google"`
- Execute: `agentd proposal add-oauth "Add OAuth with Google"`

If user input is incomplete or unclear, use AskUserQuestion to clarify:
- change-id should be kebab-case (e.g., add-oauth, fix-login-bug)
- description should be a one-sentence description of the change

## Execute Command

```bash
agentd proposal <change-id> "<description>"
```

## After Success

Report:
- Generated files location: `agentd/changes/<change-id>/`
- Suggest next step: `/agentd:challenge <change-id>`
