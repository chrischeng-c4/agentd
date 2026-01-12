---
name: specter-fix
description: Fix issues found during verification
user-invocable: true
---

# /specter:fix

Fix code issues found during verification.

## Usage

```bash
specter fix <change-id>
```

## Example

```bash
specter fix add-oauth
```

## What it does

1. Reads `VERIFICATION.md` for failed tests and issues
2. Analyzes the root cause of each failure
3. Fixes the code to pass all tests
4. Updates `IMPLEMENTATION.md` with fix notes

## Prerequisite

Must have `VERIFICATION.md` with failures. Run `/specter:verify` first.

## Next step

Run `/specter:verify <change-id>` again to confirm fixes.

If all tests pass: `/specter:archive <change-id>` to complete.
