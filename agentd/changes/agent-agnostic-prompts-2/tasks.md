---
id: agent-agnostic-prompts-2
type: tasks
version: 2
created_at: 2026-01-22T00:00:00Z
updated_at: 2026-01-22T00:05:00Z
proposal_ref: agentd/changes/agent-agnostic-prompts-2/proposal.md
summary:
  total: 13
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 13
layers:
  data:
    task_count: 2
  logic:
    task_count: 3
  integration:
    task_count: 5
  testing:
    task_count: 3
---

# Tasks

## 1. Data Layer

- [ ] 1.1 Create `UnifiedPrompt` data model
  - File: `src/models/prompt.rs` (CREATE)
  - Spec: `specs/unified-prompt-system.md#unifiedprompt-data-model`
  - Do: Define `UnifiedPrompt`, `ContextItem`, and `ContextType` enums/structs. Implement `to_xml()` method for generating standardized XML structure including `<system_prompt>`, `<context>`, and `<instructions>` tags.
  - Depends: none

- [ ] 1.2 Register `prompt` module
  - File: `src/models/mod.rs` (MODIFY)
  - Spec: `specs/unified-prompt-system.md#unifiedprompt-data-model`
  - Do: Add `pub mod prompt;` and re-export relevant types.
  - Depends: 1.1

## 2. Logic Layer

- [ ] 2.1 Implement template loading and variable interpolation
  - File: `src/orchestrator/prompts.rs` (MODIFY)
  - Spec: `specs/system-prompt-templates.md#interfaces`
  - Do: Implement `load_system_template` to load Markdown templates with agent/phase fallback logic and `inject_variables` to replace `{{VARIABLE}}` placeholders.
  - Depends: none

- [ ] 2.2 Implement centralized marker parsing utilities
  - File: `src/orchestrator/mod.rs` (MODIFY)
  - Spec: `specs/agent-orchestration-update.md#structured-response-parsing-utilities`
  - Do: Implement `parse_review_marker` returning a unified `ReviewResult` enum (Pass, NeedsRevision, NeedsChanges, Rejected, MajorIssues) and `parse_task_status`. Ensure it returns an error if markers are missing.
  - Depends: none

- [ ] 2.3 Centralize prompt assembly logic
  - File: `src/orchestrator/prompts.rs` (MODIFY)
  - Spec: `specs/agent-orchestration-update.md#centralized-prompt-assembly-logic`
  - Do: Refactor existing prompt generation functions to use `UnifiedPrompt` and the new template system.
  - Depends: 1.1, 2.1

## 3. Integration Layer

- [ ] 3.1 Initialize system prompt templates
  - File: `templates/system/` (CREATE)
  - Spec: `specs/system-prompt-templates.md#requirements`
  - Do: Create the `templates/system/` directory and populate it with `BASE-[PHASE].md` and `[AGENT]-[PHASE].md` templates (GEMINI, CLAUDE, CODEX) for all phases. Ensure templates include instructions for `<review>`, `<task_status>`, and `<thought>` tags.
  - Depends: none

- [ ] 3.2 Update Gemini orchestrator to agent-agnostic flow
  - File: `src/orchestrator/gemini.rs` (MODIFY)
  - Spec: `specs/agent-orchestration-update.md#adoption-of-unifiedprompt-model`
  - Do: Remove session-based logic (e.g., `find_session_index`) and CLI flags (`--resume`). Integrate `UnifiedPrompt`.
  - Depends: 1.1, 2.2, 2.3

- [ ] 3.3 Update Claude orchestrator to agent-agnostic flow
  - File: `src/orchestrator/claude.rs` (MODIFY)
  - Spec: `specs/agent-orchestration-update.md#adoption-of-unifiedprompt-model`
  - Do: Integrate `UnifiedPrompt` and remove session/resume logic. Remove agent-specific environment variable system prompts.
  - Depends: 1.1, 2.2, 2.3

- [ ] 3.4 Update Codex orchestrator to agent-agnostic flow
  - File: `src/orchestrator/codex.rs` (MODIFY)
  - Spec: `specs/agent-orchestration-update.md#adoption-of-unifiedprompt-model`
  - Do: Integrate `UnifiedPrompt` and remove any session persistence logic.
  - Depends: 1.1, 2.2, 2.3

- [ ] 3.5 Refactor orchestrator orchestration logic
  - File: `src/orchestrator/mod.rs` (MODIFY)
  - Spec: `specs/agent-orchestration-update.md#centralized-prompt-assembly-logic`
  - Do: Ensure the common execution path uses the new `UnifiedPrompt` and parsing utilities. Map the `ReviewResult` to the appropriate domain verdict (Challenge/Review) where needed.
  - Depends: 3.2, 3.3, 3.4

## 4. Testing Layer

- [ ] 4.1 Unit tests for `UnifiedPrompt` rendering
  - File: `src/models/prompt.rs` (MODIFY)
  - Spec: `specs/unified-prompt-system.md#acceptance-criteria`
  - Do: Add comprehensive unit tests for `to_xml()` ensuring correct tag nesting and content escaping.
  - Depends: 1.1

- [ ] 4.2 Integration tests for template system
  - File: `tests/template_test.rs` (CREATE)
  - Spec: `specs/system-prompt-templates.md#acceptance-criteria`
  - Do: Verify dynamic template loading, fallback to BASE, and interpolation of project context.
  - Depends: 2.1, 3.1

- [ ] 4.3 Orchestrator verification tests
  - File: `tests/orchestrator_verification_test.rs` (CREATE)
  - Spec: `specs/agent-orchestration-update.md#acceptance-criteria`
  - Do: Verify that orchestrators correctly handle the agent responses and parse markers correctly.
  - Depends: 3.5