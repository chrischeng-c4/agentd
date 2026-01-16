---
name: agentd:plan
description: Planning workflow (proposal and challenge)
user-invocable: true
---

# /agentd:plan

Orchestrates the entire planning phase, automatically handling proposal generation, challenge analysis, and refinement based on the current state.

## IMPORTANT: Your role is orchestration only

**DO NOT explore the codebase yourself.** Your job is to:
1. Clarify the user's requirements if ambiguous
2. Convert user intent into proper prompts/arguments
3. Run the `agentd proposal` command

The actual codebase exploration and analysis is done by:
- **Gemini** (proposal generation - 2M context window)
- **Codex** (challenge/code review)

You are a dispatcher, not an explorer.

## Git Workflow (New Changes)

For **new** changes (no existing `STATE.yaml`), ask user's preferred workflow:

1. **New branch** - `git checkout -b agentd/<change-id>`
2. **New worktree** - `git worktree add -b agentd/<change-id> ../<project>-agentd/<change-id>`
3. **In place** - Stay on current branch (default)

Skip if change already exists.

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
