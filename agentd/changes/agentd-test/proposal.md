---
id: agentd-test
type: proposal
version: 1
created_at: 2026-01-20T10:00:00Z
updated_at: 2026-01-20T10:00:00Z
author: gemini
status: proposed
iteration: 1
summary: "Test the unified plan workflow"
impact:
  scope: patch
  affected_files: 4
  new_files: 1
affected_specs:
  - id: unified-plan-test
    path: specs/unified-plan-test.md
---

<proposal>

# Change: agentd-test

## Summary

Test the unified plan workflow to verify sequential generation and challenge-reproposal orchestration.

## Why

This change is necessary to ensure that the Agentd plan workflow correctly handles multi-phase sequential generation (proposal -> specs -> tasks). By creating a controlled test case with defined affected specs, we can verify that Phase 2 and Phase 3 are correctly triggered and that the challenge-reproposal loop functions as intended in the unified architecture.

## What Changes

- Create a dummy proposal for `agentd-test` that requires at least one specification.
- Include `unified-plan-test` in the `affected_specs` list to verify Phase 2 (Spec Generation).
- Provide a summary and why section that meet the minimum character requirements (10 and 50 respectively).
- Verify that the resulting `tasks.md` correctly references the generated spec.

## Impact

- **Scope**: patch
- **Affected Files**: ~4
- **New Files**: ~1
- **Affected Specs**:
  - `unified-plan-test`
- **Affected Code**:
  - `agentd/changes/agentd-test/`

</proposal>
