# Changelog

All notable changes to Agentd will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New high-level workflow skills: `/agentd:plan`, `/agentd:impl`, `/agentd:archive`
- Phase-only state machine: workflows determine actions based on `STATE.yaml` phase
- New `rejected` phase for fundamentally flawed proposals
- Challenge command now automatically updates phase based on verdict (APPROVED → challenged, REJECTED → rejected)
- **Self-Review Workflow**: The proposal generation process now includes an automated self-review step. Gemini critiques its own PRD, Specs, and Tasks immediately after generation and performs revisions if necessary.
- **Session ID Persistence**: The `session_id` from Gemini interactions is now captured and stored in `STATE.yaml`, enabling reliable session resumption.
- **Resume-by-Index**: Added support for resuming specific session indices. The orchestrator now looks up the correct index using the stored `session_id` before resuming, replacing the fragile `--resume latest` approach.
- **Non-Zero Exit Codes**: The `agentd` CLI now exits with a non-zero status code (1) when validation fails, challenges are rejected, or max iterations are reached, improving integration with CI/CD pipelines.

### Changed
- Simplified workflow: `plan → impl → archive` replaces granular skill invocations
- Updated CLAUDE.md template with new workflow table
- **Proposal Command**: Updated to execute the self-review prompt and handle `<review>` markers (`PASS` or `NEEDS_REVISION`).
- **Gemini Orchestration**: All Gemini-based commands (proposal, reproposal, self-review) now use explicit session resumption via index.
- **CLI Mapper**: Refactored `ResumeMode` to support `Index(u32)` in addition to `Latest` and `None`.

### Fixed
- **Session Crosstalk**: Eliminated the risk of resuming incorrect sessions by enforcing explicit session ID matching.
- **Failure Detection**: Fixed the "silent failure" issue where the CLI would return success `Ok(())` even when the proposal was rejected or failed validation.

### Removed
- `testing` phase (use `implementing` instead)
- Granular skills removed: `/agentd:proposal`, `/agentd:challenge`, `/agentd:reproposal`, `/agentd:implement`, `/agentd:review`, `/agentd:resolve-reviews`, `/agentd:fix`
- Now only 3 high-level workflow skills are installed: `/agentd:plan`, `/agentd:impl`, `/agentd:archive`

## [0.1.0] - 2026-01-12

### Added
- Initial release of Agentd
- Core CLI commands: proposal, challenge, reproposal, implement, verify, archive
- Utility commands: init, list, status, refine
- AI orchestration via external scripts (Gemini, Codex, Claude)
- Interactive progress indicators
- Change management with phases (Proposed, Challenged, Implementing, Testing, Complete, Archived)
- Configuration system with `.agentd/config.toml`
- Script runner with progress feedback
- Rust-based implementation for performance and reliability

### Key Features
- **Challenge Phase**: Codex-powered proposal review
- **Iterative Refinement**: proposal → challenge → reproposal loop
- **Cost-Effective**: 70-75% cost reduction vs pure Claude approach
- **Type-Safe**: Rust compile-time guarantees
- **Single Binary**: No runtime dependencies

[Unreleased]: https://github.com/your-repo/agentd/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-repo/agentd/releases/tag/v0.1.0