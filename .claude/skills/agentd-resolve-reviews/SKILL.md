---
name: agentd:resolve-reviews
description: Fix issues found during code review
user-invocable: true
---

# /agentd:resolve-reviews

Fix code issues found during code review.

## Usage

```bash
agentd resolve-reviews <change-id>
```

## Example

```bash
agentd resolve-reviews add-oauth
```

## What it does

1. Reads `REVIEW.md` for failed tests and issues
2. Analyzes the root cause of each failure
3. Fixes the code to pass all tests
4. Updates `IMPLEMENTATION.md` with fix notes

## Prerequisite

Must have `REVIEW.md` with issues. Run `/agentd:review` first.

## Next step

Run `/agentd:review <change-id>` again to confirm fixes.

If all tests pass: `/agentd:archive <change-id>` to complete.
