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

## State transitions

```
challenged → implementing → complete
           ↗ (NEEDS_FIX - auto-fix)
```

## Next steps

- **If complete**: Run `/agentd:archive <change-id>` to archive the change
- **If failed**: Review `IMPLEMENTATION.md` and `REVIEW.md` for errors

## Error: ChangeNotReady

This error occurs when trying to implement before the proposal is approved:

```
❌ ChangeNotReady: Change must be in 'challenged' or 'implementing' phase

Current phase: proposed
Action required: Complete planning first with /agentd:plan <change-id>
```

**Resolution**: Complete the planning workflow first using `/agentd:plan`.
