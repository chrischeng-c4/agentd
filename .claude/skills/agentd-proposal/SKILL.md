---
name: agentd:proposal
description: Generate proposal using Gemini (2M context)
user-invocable: true
---

# /agentd:proposal

Generate spec-driven proposal with PRD, Technical Design, and Tickets.

## Usage

```bash
agentd proposal <change-id> "<description>"
```

## Example

```bash
agentd proposal add-oauth "Add OAuth authentication with Google and GitHub"
```

## What it does

1. Creates `changes/<change-id>/` directory
2. Calls Gemini to explore codebase and generate:
   - `proposal.md` - PRD: Why, what, impact
   - `specs/*.md` - TD: Mermaid diagrams, JSON Schema, Pseudo code, Acceptance Criteria
   - `tasks.md` - Tickets: File paths, actions, spec references, dependencies
3. Reports results

**Note**: NO actual implementation code is generated - only abstractions.

## Next step

Run `/agentd:challenge <change-id>` to analyze the proposal.
