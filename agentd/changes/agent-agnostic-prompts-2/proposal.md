---
id: agent-agnostic-prompts-2
type: proposal
version: 2
created_at: 2026-01-22T00:00:00Z
updated_at: 2026-01-22T00:05:00Z
author: mcp
status: proposed
iteration: 2
summary: "Redesign the prompt system to be agent-agnostic using structured XML input and phase-specific system templates."
impact:
  scope: major
  affected_files: 10
  new_files: 5
affected_specs:
  - id: unified-prompt-system
    path: specs/unified-prompt-system.md
  - id: agent-orchestration-update
    path: specs/agent-orchestration-update.md
  - id: system-prompt-templates
    path: specs/system-prompt-templates.md
---

<proposal>

# Change: agent-agnostic-prompts-2

## Summary

Redesign the prompt system to be agent-agnostic using structured XML input and phase-specific system templates.

## Why

The current prompt system is fragmented and agent-specific, making it hard to maintain consistency. Moving to a unified XML structure and phase-specific templates will improve reliability, simplify cross-agent workflows, and make the system truly agent-agnostic as per the SDD philosophy.

## What Changes

- Create `templates/system/` directory with phase-specific system prompts (GEMINI-plan.md, etc.)
- Implement `UnifiedPrompt` struct with `to_xml()` method for consistent XML formatting across agents
- Update orchestrators to prepend system prompts to user prompts instead of using CLI-specific flags
- Remove session reuse requirement across all orchestrators to support artifact-based context passing
- Refactor `src/orchestrator/prompts.rs` to support the new unified prompting structure

## Impact

- **Scope**: major
- **Affected Files**: ~10
- **New Files**: ~5
- Affected specs: `unified-prompt-system`, `agent-orchestration-update`, `system-prompt-templates`
- Affected code: `src/orchestrator/prompts.rs`, `src/orchestrator/gemini.rs`, `src/orchestrator/claude.rs`, `src/orchestrator/codex.rs`, `src/orchestrator/mod.rs`, `templates/system/`
- **Breaking Changes**: Removes session reuse requirement; agents now handle their own multi-turn conversations via prepended context/artifacts. System prompts move from CLI flags/env vars to prepended text.

## History & Clarifications

This proposal incorporates clarifications and requirements from the previous iteration (`agent-agnostic-prompts`) and restarts the review cycle for `agent-agnostic-prompts-2` due to missing metadata in the previous run.

- **Agent Scope**: XML structured input will support all agents (Gemini, Codex, Claude Code).
- **System Prompt Injection**: System prompts will be prepended to the user prompt to ensure consistency and avoid reliance on CLI-specific flags.
- **Session Reuse**: Explicit session reuse requirements are removed. Agents will manage context via multi-turn artifacts passed in the prompt, simplifying the architecture.
- **Git Workflow**: Development will occur on a new branch `agentd/agent-agnostic-prompts`.

</proposal>

<review iteration="2" reviewer="codex" status="needs_revision">
## Summary
Previous review issues were not found in the change artifacts, so I cannot verify whether they were addressed in this update.

## Issues
- Unable to locate the prior review findings for `agent-agnostic-prompts-2`, so resolution status cannot be confirmed.

## Verdict
needs_revision

## Next Steps
Provide the prior review findings (or point to the file/section where they live) so I can verify that each issue is resolved.
</review>
