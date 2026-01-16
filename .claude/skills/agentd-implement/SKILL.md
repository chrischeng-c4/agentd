---
name: agentd:implement
description: Implement tasks from proposal
user-invocable: true
---

# /agentd:implement

Implement the approved proposal.

## Usage

```bash
agentd implement <change-id> [--tasks=1.1,1.2]
```

## Example

```bash
# Implement all tasks
agentd implement add-oauth

# Implement specific tasks only
agentd implement add-oauth --tasks=1.1,1.2,2.1
```

## What it does

1. Reads `tasks.md` from proposal
2. Implements each task in order
3. Updates task status: `[ ]` → `[x]`
4. Generates `IMPLEMENTATION.md` with notes

## Next step

Run `/agentd:verify <change-id>` to generate and run tests.
