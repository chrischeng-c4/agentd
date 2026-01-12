---
name: specter-challenge
description: Challenge proposal with Codex analysis
user-invocable: true
---

# /specter:challenge

Analyze proposal against existing codebase.

## Usage

```bash
specter challenge <change-id>
```

## Example

```bash
specter challenge add-oauth
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

If issues found: `/specter:reproposal <change-id>` to fix automatically.

If looks good: `/specter:implement <change-id>` to start implementation.
