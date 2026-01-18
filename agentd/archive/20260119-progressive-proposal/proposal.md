---
id: progressive-proposal
type: proposal
version: 5
status: proposed
iteration: 5
summary: Add self-review prompts, explicit session resume, and non-zero exit on failure to the proposal workflow.
impact:
  scope: minor
  affected_files: 11
  new_files: 0
affected_specs:
  - id: workflow
    path: specs/workflow.md
---

# Change: progressive-proposal

## Summary
Enhance the existing progressive proposal workflow with three improvements: self-review prompts, explicit session ID resume, and non-zero exit on validation failure.

## Why
The current workflow generates PRD/Specs/Tasks progressively but lacks:
1. **Self-review**: Gemini doesn't critique its own output before validation, leading to more validation iterations
2. **Session robustness**: Using `--resume latest` can resume wrong sessions in concurrent environments
3. **Clear failure signals**: The command returns `Ok(())` even on failure, making it hard to detect issues in CI/scripts

## What Changes
- **Self-Review Prompts**: After proposal generation completes (which creates PRD, Specs, and Tasks files together), run a separate self-review prompt in the same session. If issues found, Gemini edits the files directly and outputs `<review>NEEDS_REVISION</review>`. Otherwise, outputs `<review>PASS</review>`. The orchestrator only needs to detect these markers for logging.
- **Resume-by-Index**: Add `session_id` field to State model and `UsageMetrics`. Parse session UUID from Gemini's `{"type":"init","session_id":"..."}` output and store in STATE.yaml. Before every Gemini resume (including self-review and reproposal fixes), lookup index via `--list-sessions` and use `--resume <index>` instead of `--resume latest`. Challenge phase uses Codex and is not affected.
- **Non-Zero Exit**: When validation fails after max iterations, exit with status code 1 instead of returning `Ok(())`.

## Impact
- **Affected specs**: `specs/workflow.md` (updated to reflect new behavior).
- **Affected code**:
    - `src/models/frontmatter.rs`: Add `session_id` field to State struct.
    - `src/state/manager.rs`: Add setter method to persist `session_id`.
    - `agentd/schemas/state.schema.json`: Add `session_id` property to state schema.
    - `src/orchestrator/script_runner.rs`: Add `session_id` to UsageMetrics and parse from Gemini output; ensure all Gemini commands use project_root as cwd.
    - `src/orchestrator/prompts.rs`: Add self-review prompt template for reviewing all proposal files.
    - `src/orchestrator/gemini.rs`: Add session lookup by UUID and detect self-review markers.
    - `src/orchestrator/cli_mapper.rs`: Support `ResumeMode` enum with `--resume <index>`.
    - `src/orchestrator/codex.rs`: Adapt to new `ResumeMode` API (no behavioral change).
    - `src/orchestrator/claude.rs`: Adapt to new `ResumeMode` API (no behavioral change).
    - `src/cli/proposal.rs`: Integrate self-review, use resume-by-index for Gemini calls, exit non-zero on failure.
    - `src/cli/reproposal.rs`: Use resume-by-index for Gemini reproposal calls.
- **Breaking changes**: No. The CLI interface remains identical.
