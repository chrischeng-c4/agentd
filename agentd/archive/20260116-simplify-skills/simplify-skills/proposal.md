# Change: simplify-skills

## Summary

Consolidate existing granular Agentd skills into three high-level workflows: `plan`, `impl`, and `archive`. These workflows will automatically determine the next logical action based on the current state of the change as recorded in `STATE.yaml`.

## Why

The current Agentd workflow requires users to manually invoke specific skills for each step of the process (e.g., `proposal`, `challenge`, `reproposal`, `resolve-reviews`). This granularity increases the mental overhead for the user and makes the process feel repetitive. 

By consolidating these into higher-level workflows (`plan`, `impl`, `archive`), we can provide a more seamless experience where the AI "knows" what to do next based on the project's state, reducing friction and improving efficiency.

## What Changes

- **New Skills**:
  - `agentd:plan`: Handles the entire planning cycle, from initial proposal generation to challenge analysis and refinement (reproposal).
  - `agentd:impl`: Orchestrates the implementation phase, including task execution and iterative review/resolution.
  - `agentd:archive`: Manages the archival of completed changes.
- **Phase-Only State Machine**:
  - Workflow commands only check `phase` in STATE.yaml (not verdict).
  - Challenge command updates phase based on verdict:
    - APPROVED → `phase: challenged`
    - NEEDS_REVISION → `phase: proposed` (stays for auto-reproposal)
    - REJECTED → `phase: rejected`
  - Add new `rejected` phase for fundamental issues requiring manual intervention.
- **State Transition Updates**:
  - Modify `agentd challenge` to update STATE.yaml phase based on verdict.
  - Modify `agentd archive` to set `phase: archived` on completion.
- **Skill Consolidation & Deprecation**:
  - Granular skills like `agentd:proposal`, `agentd:challenge`, `agentd:reproposal`, `agentd:implement`, `agentd:review`, and `agentd:resolve-reviews` will be deprecated from direct user invocation.
  - The `agentd:plan`, `agentd:impl`, and `agentd:archive` skills will become the primary entry points for users.
  - Administrative skills like `agentd:list`, `agentd:status`, and `agentd:init` remain available for system-wide operations.

## Impact

- Affected specs: `specs/workflows.md` (Updated for consistency)
- Affected code: 
  - `templates/skills/`: Create new skills and deprecate old ones.
  - `src/cli/init.rs`: Add new skill templates to the initialization list.
  - `.claude/skills/`: Sync with templates.
- Breaking changes:
  - User-facing skills are changing. Migration involves switching to `/agentd:plan`, `/agentd:impl`, `/agentd:archive`.
  - `testing` phase is deprecated and removed. Use `implementing` instead.