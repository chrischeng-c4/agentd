---
name: specter-verify
description: Generate and run tests
user-invocable: true
---

# /specter:verify

Verify implementation with automated tests.

## Usage

```bash
specter verify <change-id>
```

## Example

```bash
specter verify add-oauth
```

## What it does

1. Reads specs and implementation
2. Calls Codex to generate tests for each scenario
3. Runs all tests
4. Generates `VERIFICATION.md` with:
   - Test results (✅ PASS / ❌ FAIL)
   - Coverage statistics
   - Issues found

## Next step

If tests pass: `/specter:archive <change-id>` to complete.

If tests fail: Fix issues and run `/specter:verify` again.
