---
change: plan-change-idempotent
date: 2026-01-23
---

# Clarifications

## Q1: Conflict Logic
- **Question**: How should we handle the resolve_change_id_conflict logic?
- **Answer**: Remove entirely
- **Rationale**: Caller (plan.rs) decides if it's new vs continue. No auto-suffix like -2. This eliminates the root cause of duplicate change IDs being created.

## Q2: Re-run Behavior
- **Question**: What should happen when user runs plan-change on an existing change with all phases complete?
- **Answer**: Skip all, just validate
- **Rationale**: Idempotent design - if all outputs exist, only run final validation. This allows safe re-runs without side effects.

## Q3: Function Naming
- **Question**: Should we keep the function names or rename them?
- **Answer**: Rename to run_plan_change
- **Rationale**: Clearer intent, matches the command name 'agentd plan-change'. Improves code readability.

