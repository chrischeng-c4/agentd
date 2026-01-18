# Changelog

## [Unreleased]

### Added
- **Self-Review Workflow**: The proposal generation process now includes an automated self-review step. Gemini critiques its own PRD, Specs, and Tasks immediately after generation and performs revisions if necessary.
- **Session ID Persistence**: The `session_id` from Gemini interactions is now captured and stored in `STATE.yaml`, enabling reliable session resumption.
- **Resume-by-Index**: Added support for resuming specific session indices. The orchestrator now looks up the correct index using the stored `session_id` before resuming, replacing the fragile `--resume latest` approach.
- **Non-Zero Exit Codes**: The `agentd` CLI now exits with a non-zero status code (1) when validation fails, challenges are rejected, or max iterations are reached, improving integration with CI/CD pipelines.

### Changed
- **Proposal Command**: Updated to execute the self-review prompt and handle `<review>` markers (`PASS` or `NEEDS_REVISION`).
- **Gemini Orchestration**: All Gemini-based commands (proposal, reproposal, self-review) now use explicit session resumption via index.
- **CLI Mapper**: Refactored `ResumeMode` to support `Index(u32)` in addition to `Latest` and `None`.

### Fixed
- **Session Crosstalk**: Eliminated the risk of resuming incorrect sessions by enforcing explicit session ID matching.
- **Failure Detection**: Fixed the "silent failure" issue where the CLI would return success `Ok(())` even when the proposal was rejected or failed validation.
