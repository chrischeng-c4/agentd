---
id: progressive-proposal
type: tasks
version: 5
proposal_ref: proposal.md
---

# Tasks

## 1. Data Layer

- [ ] 1.1 Add session_id field to State model
  - File: `src/models/frontmatter.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Add `pub session_id: Option<String>` field to State struct with `#[serde(default)]` for backwards compatibility.
  - Depends: none

- [ ] 1.2 Add session_id to UsageMetrics
  - File: `src/orchestrator/script_runner.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Add `pub session_id: Option<String>` field to UsageMetrics. Parse `{"type":"init","session_id":"..."}` from Gemini stream-json output and extract the session_id into this field.
  - Depends: none

- [ ] 1.3 Add session_id setter to StateManager
  - File: `src/state/manager.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Add `pub fn set_session_id(&mut self, session_id: String)` method to set the session_id and mark state as dirty. The private state field prevents direct mutation, so a setter is required.
  - Depends: 1.1

- [ ] 1.4 Update state schema for session_id
  - File: `agentd/schemas/state.schema.json` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Add `session_id` property with `type: ["string", "null"]` for backwards compatibility with existing STATE.yaml files that don't have this field.
  - Depends: 1.1

## 2. Logic Layer

- [ ] 2.1 Add self-review prompt
  - File: `src/orchestrator/prompts.rs` (MODIFY)
  - Spec: specs/workflow.md (R1: Self-Review)
  - Do: Add `proposal_self_review_prompt(change_id)` that asks Gemini to review all generated proposal files (PRD, Specs, Tasks) against quality criteria, edit files if needed, and output `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`.
  - Depends: none

- [ ] 2.2 Detect self-review markers from stream-json
  - File: `src/orchestrator/gemini.rs` (MODIFY)
  - Spec: specs/workflow.md (R1: Self-Review)
  - Do: Add function to parse stream-json output, extract text from assistant messages, and detect `<review>PASS</review>` or `<review>NEEDS_REVISION</review>`. Parse `content` as `serde_json::Value` to handle both string and array-of-parts formats (Gemini can emit nested `text` fields). Handle escaped characters. If no marker found, log warning and return PASS (fallback behavior).
  - Depends: 2.1

- [ ] 2.3 Ensure all Gemini commands use project_root as cwd
  - File: `src/orchestrator/script_runner.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Modify `run_llm` and add `run_gemini_command` to accept a `project_root` parameter and set `current_dir` for ALL Gemini CLI invocations. This ensures session lookup is project-scoped regardless of where `agentd` is invoked from.
  - Depends: none

- [ ] 2.3b Implement session lookup by UUID
  - File: `src/orchestrator/gemini.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Add function `find_session_index(uuid: &str) -> Result<u32>` that calls the raw command runner with `--list-sessions`, parses output per documented format (1-indexed, UUID in brackets), and returns the index. Error handling: (1) If command fails (non-zero exit), return error with stderr/stdout. (2) If output format unexpected (no header, no UUID brackets), return error "Failed to parse session list" with raw output. (3) If UUID not found, return error "Session not found, please re-run proposal".
  - Depends: 2.3

- [ ] 2.4 Redesign CLI mapper resume API
  - File: `src/orchestrator/cli_mapper.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Replace `resume: bool` with `resume_mode: ResumeMode` enum (None, Latest, Index(u32)). Only Gemini needs `Index(n)` support; Codex/Claude continue using `Latest` when resume is needed (no changes to their behavior).
  - Depends: 2.3b

- [ ] 2.4b Update Gemini orchestrator for resume mode
  - File: `src/orchestrator/gemini.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Update all Gemini calls to use the new `ResumeMode` enum instead of `resume: bool`. Pass `ResumeMode::Index(n)` after calling `find_session_index()`.
  - Depends: 2.4

- [ ] 2.4c Update Codex/Claude orchestrators for resume mode
  - File: `src/orchestrator/codex.rs`, `src/orchestrator/claude.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Update to use `ResumeMode::Latest` or `ResumeMode::None` as appropriate. No behavioral change - just adapt to new API signature.
  - Depends: 2.4

## 3. Integration

- [ ] 3.0 Save session_id to STATE.yaml after generation
  - File: `src/cli/proposal.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: After proposal generation, extract `usage.session_id` from UsageMetrics and save via `StateManager::set_session_id()`. If None, log "Failed to capture session ID" and exit with status 1 before any self-review or resume-by-index.
  - Depends: 1.2, 1.3

- [ ] 3.1 Integrate self-review into workflow
  - File: `src/cli/proposal.rs` (MODIFY)
  - Spec: specs/workflow.md (R1: Self-Review)
  - Do: After proposal generation, call the self-review prompt. Detect the marker to log whether revisions were made (Gemini edits files directly).
  - Depends: 2.1, 2.2, 3.0

- [ ] 3.2 Use resume-by-index in all Gemini calls
  - File: `src/cli/proposal.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Replace ALL Gemini `--resume latest` usages with `--resume <index>`. This includes: self-review calls and reproposal fix loops. Challenge uses Codex (not affected). Call `find_session_index()` before each Gemini resume.
  - Depends: 2.3b, 2.4, 2.4b, 3.0

- [ ] 3.3 Exit non-zero on failure
  - File: `src/cli/proposal.rs` (MODIFY)
  - Spec: specs/workflow.md (R3: Non-Zero Exit)
  - Do: Replace `return Ok(())` with `std::process::exit(1)` in these scenarios: (1) format validation fails after `format_iterations` attempts, (2) challenge `NEEDS_REVISION` still failing after `planning_iterations`, (3) challenge `REJECTED` (immediate exit), (4) session ID capture/lookup fails.
  - Depends: none

- [ ] 3.4 Update reproposal CLI to use resume-by-index
  - File: `src/cli/reproposal.rs` (MODIFY)
  - Spec: specs/workflow.md (R2: Resume-by-Index)
  - Do: Update `agentd reproposal` command to read session_id from STATE.yaml, call `find_session_index()`, and use `--resume <index>` instead of `--resume latest`.
  - Depends: 1.3, 2.3b, 2.4b

- [ ] 3.5 Add explicit logging for self-review outcome
  - File: `src/cli/proposal.rs` (MODIFY)
  - Spec: specs/workflow.md (R1: Self-Review)
  - Do: After detecting self-review marker, log a clear message: "Self-review: PASS (no changes)" or "Self-review: NEEDS_REVISION (files updated)".
  - Depends: 3.1

## 4. Testing

- [ ] 4.1 Test self-review marker parsing
  - File: `src/orchestrator/gemini.rs` (TEST)
  - Verify: specs/workflow.md (R1: Self-Review)
  - Do: Test marker detection for PASS/NEEDS_REVISION, including edge cases: (a) escaped characters in stream-json, (b) multi-line content around markers, (c) no marker found (should return PASS with warning).
  - Depends: 2.2

- [ ] 4.2 Test session index lookup
  - File: `src/orchestrator/gemini.rs` (TEST)
  - Verify: specs/workflow.md (R2: Resume-by-Index)
  - Do: Test happy path (UUID found returns index), and negative paths: (a) command failure propagation, (b) unparseable output format, (c) UUID not found in list.
  - Depends: 2.3b

- [ ] 4.3 Test CLI mapper resume-by-index
  - File: `src/orchestrator/cli_mapper.rs` (TEST)
  - Verify: specs/workflow.md (R2: Resume-by-Index)
  - Do: Test that `ResumeMode::Index(5)` produces `--resume 5` for Gemini, and other providers continue working.
  - Depends: 2.4

- [ ] 4.4 Test session_id extraction and persistence
  - File: `src/orchestrator/script_runner.rs`, `src/state/manager.rs` (TEST)
  - Verify: specs/workflow.md (R2: Resume-by-Index)
  - Do: Test parsing `{"type":"init","session_id":"..."}` from stream-json, and test StateManager set_session_id persists correctly across load/save cycles.
  - Depends: 1.2, 1.3
