---
name: agentd:plan
description: Planning workflow (proposal and challenge)
user-invocable: true
---

# /agentd:plan

Orchestrates the entire planning phase, automatically handling proposal generation, challenge analysis, and refinement based on the current state.

## IMPORTANT: Your role is orchestration only

**DO NOT explore the codebase yourself.** Your job is to:
1. Clarify the user's requirements (structured Q&A)
2. Write clarifications to `clarifications.md`
3. Run the `agentd proposal` command

The actual codebase exploration and analysis is done by:
- **Gemini** (proposal generation - 2M context window)
- **Codex** (challenge/code review)

You are a dispatcher, not an explorer.

## Clarification Phase (Before Proposal)

For **NEW changes** (no existing `STATE.yaml`), clarify requirements before running `agentd proposal`:

### When to clarify
- Always for new changes, unless user says "skip" or description is very detailed
- Skip for existing changes (continuing from `proposed` phase)

### How to clarify
1. Analyze the description for ambiguities
2. Use the **AskUserQuestion tool** to ask **3-5 questions max**:

```
AskUserQuestion with questions array:
- question: "What is your preferred approach for X?"
  header: "Short Label" (max 12 chars)
  options:
    - label: "Option A (Recommended)"
      description: "Why this is the best choice"
    - label: "Option B"
      description: "Alternative approach"
  multiSelect: false
```

**Important**: Always use the AskUserQuestion tool for interactive clarification, not text-based questions.

3. After user answers, write to `agentd/changes/<change-id>/clarifications.md`:

```markdown
---
change: <change-id>
date: YYYY-MM-DD
---

# Clarifications

## Q1: [Topic]
- **Question**: [the question asked]
- **Answer**: [user's answer]
- **Rationale**: [why this choice]
```

4. Then run `agentd proposal` with the clarified context

### Skip clarification if
- User explicitly says "skip" or uses `--skip-clarify`
- Description already covers all key decisions
- Continuing an existing change (phase is `proposed`)

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
| No STATE.yaml | **Clarify** → write `clarifications.md` → run `agentd proposal` |
| `proposed` | Run `agentd proposal` to continue planning cycle |
| `challenged` | ✅ Planning complete, suggest `/agentd:impl` |
| `rejected` | ⛔ Rejected, suggest reviewing CHALLENGE.md |
| Other phases | ℹ️ Beyond planning phase |

**Note**: The `agentd proposal` command internally handles challenge analysis and auto-reproposal loops. It will iterate until the proposal is either APPROVED (phase → `challenged`) or REJECTED (phase → `rejected`).

## State transitions

```
No STATE.yaml → [Clarify] → proposed → challenged  (APPROVED)
                          ↓         ↗ (NEEDS_REVISION - auto-reproposal)
                          → rejected (REJECTED)
```

## Next steps

- **If challenged**: Run `/agentd:impl <change-id>` to implement
- **If rejected**: Review `CHALLENGE.md` and fix fundamental issues manually
