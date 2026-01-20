---
name: agentd:impl
description: Implementation workflow
user-invocable: true
---

# /agentd:impl

Orchestrates the implementation phase, handling code generation, review, and iterative fixes based on the current state.

## IMPORTANT: Your role is orchestration only

**DO NOT implement code yourself.** Your job is to:
1. Check the current phase in `STATE.yaml`
2. Run the `agentd implement` command

The actual implementation is done by a **separate Claude session** spawned by the command. This session has access to the proposal specs and implements according to `tasks.md`.

You are a dispatcher, not an implementer. Run the command and let the subprocess handle the work.

## Usage

```bash
/agentd:impl <change-id>
```

## Example

```bash
/agentd:impl add-oauth
```

## How it works

The skill determines readiness based on the `phase` field in `STATE.yaml`:

| Phase | Action |
|-------|--------|
| `challenged` | ✅ Run `agentd implement` to start implementation |
| `implementing` | ✅ Continue `agentd implement` (resume or retry) |
| Other phases | ❌ **ChangeNotReady** error - not ready for implementation |

**Note**: The `agentd implement` command internally handles code review and auto-fix loops. It will iterate until all tests pass and code review is approved (phase → `complete`).

## Prerequisites

- Change must have passed challenge (phase: `challenged`)
- All planning artifacts must exist:
  - `proposal.md`
  - `tasks.md`
  - `specs/*.md`

## Knowledge Reference

Before implementation, the spawned session may consult `agentd/knowledge/` for:
- Existing patterns and conventions
- Module-specific implementation details
- Architecture constraints

Use `read_knowledge` MCP tool to access documentation.

## After `agentd implement` completes

Once implementation finishes, check the STATE.yaml phase and use **AskUserQuestion** based on the outcome:

### If phase is `complete` (Implementation Approved)

```
AskUserQuestion:
  question: "Implementation approved! What would you like to do?"
  header: "Next Action"
  options:
    - label: "Archive change (Recommended)"
      description: "Run /agentd:archive to finalize and archive the change"
    - label: "Review implementation"
      description: "Check REVIEW.md and IMPLEMENTATION.md before archiving"
    - label: "Manual testing"
      description: "Test the implementation manually before proceeding"
```

- **Archive change**: Suggest `/agentd:archive <change-id>`
- **Review implementation**: Run `agentd view <change-id>` or open REVIEW.md
- **Manual testing**: Inform user to test manually, then they can run `/agentd:archive` when ready

### If phase is still `implementing` (Auto-fix limit reached)

This means the automatic refinement iterations have been exhausted but issues remain.

```
AskUserQuestion:
  question: "Automatic refinement limit reached. Issues remain - how to proceed?"
  header: "Next Action"
  options:
    - label: "Review issues (Recommended)"
      description: "Check REVIEW.md to understand remaining problems"
    - label: "Manual fix"
      description: "Fix issues manually and re-run implementation"
    - label: "Accept and archive"
      description: "Accept current state and archive with known issues"
```

- **Review issues**: Run `agentd view <change-id>` or display REVIEW.md content
- **Manual fix**: Inform user to fix manually, then run `agentd implement <change-id>` again
- **Accept and archive**: Run `/agentd:archive <change-id>` (not recommended but possible)

### If major issues found (phase still `implementing` with critical errors)

```
AskUserQuestion:
  question: "Major implementation issues found. How would you like to proceed?"
  header: "Next Action"
  options:
    - label: "Review full report (Recommended)"
      description: "Check REVIEW.md and IMPLEMENTATION.md for details"
    - label: "Manual intervention"
      description: "Fix critical issues manually before retrying"
    - label: "Restart implementation"
      description: "Retry full implementation from scratch"
```

- **Review full report**: Display REVIEW.md summary or run `agentd view <change-id>`
- **Manual intervention**: Inform user to fix manually, then run `agentd implement <change-id>`
- **Restart implementation**: Run `agentd implement <change-id>` again

## State transitions

```
challenged → implementing → complete
           ↗ (NEEDS_FIX - auto-fix)
```

## Error: ChangeNotReady

This error occurs when trying to implement before the proposal is approved:

```
❌ ChangeNotReady: Change must be in 'challenged' or 'implementing' phase

Current phase: proposed
Action required: Complete planning first with /agentd:plan <change-id>
```

**Resolution**: Complete the planning workflow first using `/agentd:plan`.
