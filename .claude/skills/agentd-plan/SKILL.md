---
name: agentd:plan
description: Planning workflow (proposal and challenge)
user-invocable: true
---

# /agentd:plan

Orchestrates the entire planning phase, automatically handling proposal generation, challenge analysis, and refinement based on the current state.

## Usage

```bash
# New change (description required)
/agentd:plan <change-id> "<description>"

# Existing change (continue planning)
/agentd:plan <change-id>
```

## Examples

```bash
# Start new planning cycle
/agentd:plan add-oauth "Add OAuth authentication with Google and GitHub"

# Continue planning for existing change
/agentd:plan add-oauth
```

## How it works

The skill determines the next action based on the `phase` field in `STATE.yaml`:

| Phase | Action |
|-------|--------|
| No STATE.yaml | Run `agentd proposal` (description required) |
| `proposed` | Run `agentd proposal` to continue planning cycle |
| `challenged` | ✅ Planning complete, suggest `/agentd:impl` |
| `rejected` | ⛔ Rejected, suggest reviewing CHALLENGE.md |
| Other phases | ℹ️ Beyond planning phase |

**Note**: The `agentd proposal` command internally handles challenge analysis and auto-reproposal loops. It will iterate until the proposal is either APPROVED (phase → `challenged`) or REJECTED (phase → `rejected`).

## State transitions

```
No STATE.yaml → proposed → challenged  (APPROVED)
              ↓         ↗ (NEEDS_REVISION - auto-reproposal)
              → rejected (REJECTED)
```

## Next steps

- **If challenged**: Run `/agentd:impl <change-id>` to implement
- **If rejected**: Review `CHALLENGE.md` and fix fundamental issues manually
