---
id: plan-change-idempotent
type: proposal
version: 1
created_at: 2026-01-23T04:35:40.826596+00:00
updated_at: 2026-01-23T04:35:40.826596+00:00
author: mcp
status: proposed
iteration: 1
summary: "Refactor proposal_engine to make plan-change workflow idempotent"
impact:
  scope: minor
  affected_files: 3
  new_files: 0
---

<proposal>

# Change: plan-change-idempotent

## Summary

Refactor proposal_engine to make plan-change workflow idempotent

## Why

The current proposal_engine has several architectural issues:

1. **Duplicate change ID creation**: When running `agentd plan-change` on an existing change, `resolve_change_id_conflict` incorrectly triggers and creates a new ID with suffix (e.g., `my-change-2`). This happens because the function doesn't distinguish between "continuing an existing change" vs "creating a new change with a conflicting name".

2. **Redundant review loops**: `run_proposal_loop` calls `run_proposal_step_sequential` (which already has internal review loops for proposal, specs, and tasks), then runs an ADDITIONAL outer challenge/reproposal loop. This is redundant since reviews are already integrated.

3. **Non-idempotent design**: Running plan-change twice on the same change regenerates everything instead of skipping completed phases. This wastes API calls and can overwrite user edits.

4. **Confusing function names**: `run_proposal_loop` and `run_proposal_step_sequential` don't clearly convey the workflow intent.

## What Changes

- Remove `resolve_change_id_conflict` from proposal_engine - caller (plan.rs) handles new vs continue logic
- Merge `run_proposal_loop` and `run_proposal_step_sequential` into single `run_plan_change` function
- Remove outer challenge/reproposal loop (reviews already integrated in each phase)
- Add idempotent checks: skip Phase 1 if proposal.md exists, skip Phase 2 if spec exists, skip Phase 3 if tasks.md exists
- If all phases complete, only run final validation
- Update plan.rs to handle conflict detection before calling run_plan_change

## Impact

- **Scope**: minor
- **Affected Files**: ~3
- **New Files**: ~0
- Affected code: `src/cli/proposal_engine.rs - main refactoring target`, `src/cli/plan.rs - update to handle new vs continue logic`, `src/context.rs - possibly remove or simplify resolve_change_id_conflict`

</proposal>
