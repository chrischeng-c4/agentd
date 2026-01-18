---
name: agentd:archive
description: Archive workflow - merge specs and move to archive
user-invocable: true
---

# /agentd:archive

Orchestrates the archival process, merging spec deltas to main specs and moving the change to the archive directory.

## IMPORTANT: Your role is orchestration only

**DO NOT perform archival steps yourself.** Your job is to:
1. Check the current phase in `STATE.yaml`
2. Run the `agentd archive` command

The actual merging and archiving is done by the command, which uses:
- **Gemini** (spec merging)
- **Codex** (archive review)

You are a dispatcher, not an archiver. Run the command and let it handle the work.

## Usage

```bash
/agentd:archive <change-id>
```

## Example

```bash
/agentd:archive add-oauth
```

## How it works

The skill checks the `phase` field in `STATE.yaml`:

| Phase | Action |
|-------|--------|
| `complete` | ✅ Run `agentd archive` to merge and archive |
| Other phases | ❌ **ChangeNotComplete** error - not ready for archive |

## What it does

1. **Validates** spec files (zero token cost)
2. **Analyzes** delta metrics and decides merge strategy
3. **Backs up** original specs for rollback
4. **Merges** spec deltas with Gemini (applies changes to `agentd/specs/`)
5. **Generates** CHANGELOG entry
6. **Reviews** merged specs with Codex (with auto-fix loop if needed)
7. **Moves** change to `agentd/archive/YYYYMMDD-<change-id>/`
8. Updates phase to `archived` in STATE.yaml

## Prerequisites

- Change must be complete (phase: `complete`)
- All implementation artifacts must exist and pass validation

## State transitions

```
complete → archived
```

## Error: ChangeNotComplete

This error occurs when trying to archive before implementation is complete:

```
❌ ChangeNotComplete: Change must be in 'complete' phase

Current phase: implementing
Action required: Complete implementation first with /agentd:impl <change-id>
```

**Resolution**: Ensure all implementation tasks are complete and tests pass using `/agentd:impl`.
