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

3. After user answers, use the **create_clarifications MCP tool**:

```json
{
  "change_id": "<change-id>",
  "questions": [
    {
      "topic": "Short Label",
      "question": "What is your preferred approach for X?",
      "answer": "User's answer from AskUserQuestion",
      "rationale": "Why this choice makes sense"
    }
  ]
}
```

The tool will:
- Create `agentd/changes/<change-id>/` directory if needed
- Write `clarifications.md` with proper frontmatter
- Return success message

4. Then run `agentd plan` with the clarified context

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

# Existing change (description optional)
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
| No STATE.yaml | **Clarify** → write `clarifications.md` → run `agentd plan` |
| `proposed` | Run `agentd plan` to continue planning cycle |
| `challenged` | ✅ Planning complete, suggest `/agentd:impl` |
| `rejected` | ⛔ Rejected, suggest reviewing CHALLENGE.md |
| Other phases | ℹ️ Beyond planning phase |

**Note**: The `agentd plan` command uses **Human-in-the-Loop (HITL)** mode by default:
- Generates proposal → specs → tasks sequentially with fresh sessions
- Runs challenge with Codex
- **Stops and waits for human decision** (no auto-reproposal loop)

## Human-in-the-Loop Flow

After `agentd plan` completes, check the challenge verdict and use **AskUserQuestion** to let user decide:

### If verdict is APPROVED:
- ✅ Planning complete
- Suggest: `/agentd:impl <change-id>`

### If verdict is NEEDS_REVISION:
Use AskUserQuestion with these options:

```
AskUserQuestion:
  question: "The proposal needs revision. What would you like to do?"
  header: "Next Action"
  options:
    - label: "Auto-fix and rechallenge (Recommended)"
      description: "Run agentd reproposal to fix issues, then rechallenge with Codex"
    - label: "Review manually"
      description: "Let me review the CHALLENGE.md and proposal files first"
    - label: "Stop here"
      description: "I'll handle this manually later"
```

**Then execute based on user choice:**
- **Auto-fix**: Run `agentd reproposal <change-id>` → then `agentd challenge <change-id>` → repeat this flow
- **Review manually**: Read CHALLENGE.md and show issues to user
- **Stop**: Exit gracefully

### If verdict is REJECTED:
- ⛔ Fundamental issues detected
- Read and display key issues from CHALLENGE.md
- Suggest manual review and fixes

## State transitions (Human-in-the-Loop)

```
No STATE.yaml → [Clarify] → proposed → [Challenge] → APPROVED → challenged ✅
                          ↓                       ↓
                          ↓                    NEEDS_REVISION
                          ↓                       ↓
                          ↓              [AskUserQuestion] ← YOU ARE HERE
                          ↓                       ↓
                          ↓            User chooses: Auto-fix / Review / Stop
                          ↓                       ↓
                          ↓              [reproposal] → [rechallenge] → loop
                          ↓
                          → REJECTED → rejected ⛔
```

## Next steps

- **If challenged**: Run `/agentd:impl <change-id>` to implement
- **If rejected**: Review `CHALLENGE.md` and fix fundamental issues manually
