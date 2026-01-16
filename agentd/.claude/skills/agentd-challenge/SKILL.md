---
name: agentd:challenge
description: Challenge proposal with Codex analysis
user-invocable: true
---

# /agentd:challenge

Analyze proposal against existing codebase.

## Usage

```bash
agentd challenge <change-id>
```

## Example

```bash
agentd challenge add-oauth
```

## What it does

1. Reads proposal from `changes/<change-id>/`
2. Calls Codex to analyze against existing code
3. Generates `CHALLENGE.md` with:
   - Issues found (HIGH/MEDIUM/LOW severity)
   - Architecture conflicts
   - Naming inconsistencies
   - Missing migration paths
4. Shows summary

## Next step

If issues found: `/agentd:reproposal <change-id>` to fix automatically.

If looks good: `/agentd:implement <change-id>` to start implementation.
