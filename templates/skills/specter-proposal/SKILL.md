---
name: specter-proposal
description: Generate proposal using Gemini (2M context)
user-invocable: true
---

# /specter:proposal

Generate spec-driven proposal.

## Usage

```bash
specter proposal <change-id> "<description>"
```

## Example

```bash
specter proposal add-oauth "Add OAuth authentication with Google and GitHub"
```

## What it does

1. Creates `changes/<change-id>/` directory
2. Calls Gemini to explore codebase and generate:
   - `proposal.md` - Why, what, impact
   - `tasks.md` - Implementation checklist
   - `diagrams.md` - Architecture diagrams
   - `specs/*/spec.md` - Requirements (WHEN/THEN format)
3. Reports results

## Next step

Run `/specter:challenge <change-id>` to analyze the proposal.
