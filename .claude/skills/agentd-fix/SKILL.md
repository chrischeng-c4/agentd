---
name: agentd:fix
description: Fix issues found during verification
user-invocable: true
---

# /agentd:fix

Fix code issues found during verification.

## Usage

```bash
agentd fix <change-id>
```

## Example

```bash
agentd fix add-oauth
```

## What it does

1. Reads `VERIFICATION.md` for failed tests and issues
2. Analyzes the root cause of each failure
3. Fixes the code to pass all tests
4. Updates `IMPLEMENTATION.md` with fix notes

## Prerequisite

Must have `VERIFICATION.md` with failures. Run `/agentd:verify` first.

## Next step

Run `/agentd:verify <change-id>` again to confirm fixes.

If all tests pass: `/agentd:archive <change-id>` to complete.
