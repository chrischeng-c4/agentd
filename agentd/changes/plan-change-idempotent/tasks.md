---
id: plan-change-idempotent
type: tasks
version: 1
created_at: 2026-01-23T04:37:15.950061+00:00
updated_at: 2026-01-23T04:37:15.950061+00:00
proposal_ref: plan-change-idempotent
summary:
  total: 7
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 7
layers:
  logic:
    task_count: 5
    estimated_files: 0
  integration:
    task_count: 1
    estimated_files: 0
  testing:
    task_count: 1
    estimated_files: 0
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 7 implementation tasks for change `plan-change-idempotent`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 5 | 🔲 Pending |
| Integration Layer | 1 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create run_plan_change function skeleton

```yaml
id: 2.1
action: MODIFY
status: pending
file: src/cli/proposal_engine.rs
spec_ref: idempotent-plan-change:R3
```

Create new `run_plan_change` function that will replace both `run_proposal_loop` and `run_proposal_step_sequential`. Start with the function signature and config struct.

### Task 2.2: Implement phase skip checks

```yaml
id: 2.2
action: MODIFY
status: pending
file: src/cli/proposal_engine.rs
spec_ref: idempotent-plan-change:R1
depends_on: [1.1]
```

Add existence checks before each phase: check proposal.md before Phase 1, check each spec file before Phase 2, check tasks.md before Phase 3. Skip phase if output exists.

### Task 2.3: Remove resolve_change_id_conflict call

```yaml
id: 2.3
action: MODIFY
status: pending
file: src/cli/proposal_engine.rs
spec_ref: idempotent-plan-change:R2
depends_on: [1.1]
```

Remove the call to `resolve_change_id_conflict` from the proposal engine. The change_id passed in is used directly without modification.

### Task 2.4: Implement validation-only path

```yaml
id: 2.4
action: MODIFY
status: pending
file: src/cli/proposal_engine.rs
spec_ref: idempotent-plan-change:R4
depends_on: [1.2]
```

When all phases are complete (all files exist), skip directly to final validation. Return early with success result without making any LLM API calls.

### Task 2.5: Remove old functions

```yaml
id: 2.5
action: MODIFY
status: pending
file: src/cli/proposal_engine.rs
spec_ref: idempotent-plan-change:R3
depends_on: [1.1, 1.2, 1.3, 1.4]
```

Remove `run_proposal_loop`, `run_proposal_step_sequential`, `run_challenge_step`, `run_rechallenge_step`, and `run_reproposal_step` functions. Update any remaining references.

## 3. Integration Layer

### Task 3.1: Update plan.rs caller logic

```yaml
id: 3.1
action: MODIFY
status: pending
file: src/cli/plan.rs
spec_ref: idempotent-plan-change:R5
depends_on: [1.5]
```

Update plan.rs to call `run_plan_change` instead of `run_proposal_loop`. Handle new vs continue logic here: check if STATE.yaml exists, if not and directory exists with proposal.md, this is a conflict.

## 4. Testing Layer

### Task 4.1: Test idempotent behavior

```yaml
id: 4.1
action: MODIFY
status: pending
file: src/cli/proposal_engine.rs
spec_ref: idempotent-plan-change:R1
depends_on: [2.1]
```

Add unit tests verifying: (1) new change runs all phases, (2) existing proposal.md skips Phase 1, (3) all files exist skips to validation only.

</tasks>
