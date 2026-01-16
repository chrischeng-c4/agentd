---
name: agentd:review
description: Generate and run tests
user-invocable: true
---

# /agentd:verify

Verify implementation with automated tests.

## Usage

```bash
agentd verify <change-id>
```

## Example

```bash
agentd verify add-oauth
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

If tests pass: `/agentd:archive <change-id>` to complete.

If tests fail: Fix issues and run `/agentd:verify` again.
