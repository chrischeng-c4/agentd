# Tasks

## 1. Data Layer (Prompts & Configuration)

- [ ] 1.1 Create prompt templates in Rust
  - File: `src/orchestrator/prompts.rs` (CREATE)
  - Spec: `specs/orchestrator.md#R1`
  - Do: Move HEREDOC prompts from shell scripts to Rust constants or using a template macro. Include pre-processing result placeholders for review.
  - Depends: none

- [ ] 1.2 Update Orchestrator structs with ModelSelector
  - File: `src/orchestrator/gemini.rs` (MODIFY)
  - File: `src/orchestrator/claude.rs` (MODIFY)
  - File: `src/orchestrator/codex.rs` (MODIFY)
  - Spec: `specs/orchestrator.md#R4`
  - Do: Add `ModelSelector` to orchestrator structs and use it in each `run_*` method to pick the model.
  - Depends: 1.1

## 2. Logic Layer (Execution & Orchestration)

- [ ] 2.1 Implement direct CLI execution logic
  - File: `src/orchestrator/script_runner.rs` (MODIFY)
  - Spec: `specs/orchestrator.md#R2`
  - Do: Implement a generic `run_cli` method that handles `tokio::process::Command`, stdin piping, and output streaming. Handle both `GEMINI_SYSTEM_MD` and `GEMINI_INSTRUCTIONS_FILE` if needed.
  - Depends: none

- [ ] 2.2 Implement GeminiOrchestrator logic
  - File: `src/orchestrator/gemini.rs` (MODIFY)
  - Spec: `specs/orchestrator.md#Interfaces`
  - Do: Implement `run_proposal`, `run_reproposal`, `run_merge_specs`, `run_changelog`, `run_fillback`, `run_archive_fix`.
  - Depends: 1.2, 2.1

- [ ] 2.3 Implement ClaudeOrchestrator logic
  - File: `src/orchestrator/claude.rs` (MODIFY)
  - Spec: `specs/orchestrator.md#Interfaces`
  - Do: Implement `run_implement`, `run_resolve`.
  - Depends: 1.2, 2.1

- [ ] 2.4 Implement CodexOrchestrator & Review Pipeline
  - File: `src/orchestrator/codex.rs` (MODIFY)
  - Spec: `specs/orchestrator.md#R6`
  - Do: Implement `run_challenge`, `run_rechallenge`, `run_review`, `run_verify`, `run_archive_review`. Add logic to execute `cargo test`, `clippy`, etc., for `run_review`.
  - Depends: 1.2, 2.1

## 3. Integration

- [ ] 3.1 Update init and config
  - File: `src/cli/init.rs` (MODIFY)
  - File: `src/models/change.rs` (MODIFY)
  - Spec: `specs/orchestrator.md#R2`
  - Do: Update `init` to stop writing `.sh` scripts. Deprecate `scripts_dir` in `AgentdConfig`.
  - Depends: none

- [ ] 3.2 Refactor CLI commands and Fillback
  - File: `src/cli/proposal.rs` (MODIFY)
  - File: `src/cli/reproposal.rs` (MODIFY)
  - File: `src/cli/implement.rs` (MODIFY)
  - File: `src/cli/challenge_proposal.rs` (MODIFY)
  - File: `src/cli/review.rs` (MODIFY)
  - File: `src/cli/resolve_reviews.rs` (MODIFY)
  - File: `src/cli/archive.rs` (MODIFY)
  - File: `src/fillback/code.rs` (MODIFY)
  - Spec: `specs/orchestrator.md#Flow`
  - Do: Replace `ScriptRunner` calls with appropriate orchestrator calls.
  - Depends: 2.2, 2.3, 2.4

- [ ] 3.3 Update documentation
  - File: `README.md` (MODIFY)
  - File: `INSTALL.md` (MODIFY)
  - Do: Update documentation to reflect direct CLI tool dependencies and any environment variable changes.
  - Depends: 3.2

- [ ] 3.4 Deprecate agentd/scripts/
  - File: `agentd/scripts/*.sh` (DELETE)
  - Do: Remove shell scripts.
  - Depends: 3.3

## 4. Testing

- [ ] 4.1 Test prompt generation
  - File: `src/orchestrator/prompts.rs` (MODIFY)
  - Verify: `specs/orchestrator.md#R1`
  - Do: Add unit tests to verify prompts are correctly parameterized.
  - Depends: 1.1

- [ ] 4.2 Test Orchestrator execution & Review Pipeline
  - File: `src/orchestrator/mod.rs` (MODIFY)
  - Verify: `specs/orchestrator.md#Acceptance Criteria`
  - Do: Add integration tests (mocking commands) to verify correct CLI invocation, stderr capture/streaming, and local tool execution.
  - Depends: 2.4
