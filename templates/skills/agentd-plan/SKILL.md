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

## After `agentd plan` completes

The proposal engine returns a result with:
- `verdict`: APPROVED, NEEDS_REVISION, or REJECTED
- `iteration_count`: Number of reproposal iterations completed
- `has_only_minor_issues`: True if only LOW severity issues remain (or at most 1 MEDIUM)

Use **AskUserQuestion** based on the verdict and context:

### If verdict is APPROVED

```
AskUserQuestion:
  question: "Proposal approved! What would you like to do?"
  header: "Next Action"
  options:
    - label: "Proceed to implementation (Recommended)"
      description: "Run /agentd:impl to start implementing the change"
    - label: "Open viewer"
      description: "Review the approved plan in the UI viewer"
```

- **Proceed to implementation**: Suggest `/agentd:impl <change-id>`
- **Open viewer**: Run `agentd view <change-id>`

### If verdict is NEEDS_REVISION

The options depend on context:

#### When `iteration_count >= planning_iterations` OR `has_only_minor_issues`:

```
AskUserQuestion:
  question: "Reproposal complete. Minor issues remain - can proceed to implementation."
  header: "Next Action"
  options:
    - label: "Proceed to implementation (Recommended)"
      description: "Minor issues can be addressed during implementation"
    - label: "Open viewer"
      description: "Review the remaining issues before deciding"
    - label: "Continue fixing"
      description: "Run another reproposal cycle to address issues"
```

- **Proceed to implementation**: Suggest `/agentd:impl <change-id>`
- **Open viewer**: Run `agentd view <change-id>`
- **Continue fixing**: Run `agentd reproposal <change-id>` then `agentd challenge <change-id>`

#### When significant issues remain (not minor):

```
AskUserQuestion:
  question: "Issues found. How would you like to proceed?"
  header: "Next Action"
  options:
    - label: "Open viewer (Recommended)"
      description: "Review the issues before deciding"
    - label: "Continue fixing"
      description: "Run another reproposal cycle"
    - label: "Proceed anyway"
      description: "Skip to implementation despite issues"
```

- **Open viewer**: Run `agentd view <change-id>`
- **Continue fixing**: Run `agentd reproposal <change-id>` then `agentd challenge <change-id>`
- **Proceed anyway**: Suggest `/agentd:impl <change-id>`

### If verdict is REJECTED

Display rejection message and suggest reviewing the review block in proposal.md:

```
The proposal was rejected due to fundamental issues.
Please review: agentd/changes/<change-id>/proposal.md

Consider:
- Revising the description and requirements
- Starting a new proposal with a different approach
```
