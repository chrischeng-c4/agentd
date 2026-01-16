# Challenge Report: move-shell-to-rust

## Summary
Proposal revisions are close, but several spec/task mismatches remain (notably review tooling, env setup, and scope), so it still needs revision before implementation.

## Internal Consistency Issues
### Issue: Spec marked "to be created" but already exists
- **Severity**: High
- **Category**: Consistency
- **Description**: The proposal claims the orchestrator spec is “to be created,” but `specs/orchestrator.md` is already present in this change set.
- **Location**: `agentd/changes/move-shell-to-rust/proposal.md` (Impact section)
- **Recommendation**: Update the Impact section to reference the existing spec or remove the “to be created” note.

### Issue: Fillback scope mismatch between proposal and tasks
- **Severity**: High
- **Category**: Consistency
- **Description**: The proposal says “Update ALL CLI commands (including fillback...)” to use new orchestrators, but tasks only mention `src/fillback/code.rs` and omit the CLI entry point `src/cli/fillback.rs`.
- **Location**: `agentd/changes/move-shell-to-rust/proposal.md` (What Changes), `agentd/changes/move-shell-to-rust/tasks.md` (3.2)
- **Recommendation**: Either add an explicit task for `src/cli/fillback.rs` or revise the proposal to exclude fillback from the orchestrator migration.

### Issue: Review pipeline tool list differs between spec and tasks
- **Severity**: High
- **Category**: Consistency
- **Description**: The spec requires semgrep as part of review pre-processing, but tasks only mention cargo test/clippy/audit. This drops a required tool from implementation tasks.
- **Location**: `agentd/changes/move-shell-to-rust/specs/orchestrator.md` (R6), `agentd/changes/move-shell-to-rust/tasks.md` (2.4)
- **Recommendation**: Add semgrep execution to tasks (and acceptance tests) or update R6 to match the intended toolset.

### Issue: Environment setup requirements not reflected in tasks
- **Severity**: High
- **Category**: Completeness
- **Description**: R3 requires `CODEX_INSTRUCTIONS_FILE` and other env setup, but tasks only call out Gemini-specific env handling. There’s no explicit work item for Codex/Claude env wiring.
- **Location**: `agentd/changes/move-shell-to-rust/specs/orchestrator.md` (R3), `agentd/changes/move-shell-to-rust/tasks.md` (2.1)
- **Recommendation**: Add explicit tasks/tests to cover Codex/Claude env configuration (e.g., `CODEX_INSTRUCTIONS_FILE`, model env vars) or clarify where this is handled.

## Code Alignment Issues
### Issue: init.rs currently embeds script files
- **Severity**: Medium
- **Category**: Conflict
- **Description**: `src/cli/init.rs` uses `include_str!` for files under `agentd/scripts/`. Removing scripts without removing these includes will break builds.
- **Location**: `src/cli/init.rs`
- **Note**: This is expected given the proposal to remove scripts, but needs explicit handling in the refactor.
- **Recommendation**: Ensure the init refactor removes/rewrites `include_str!` references alongside script deletion.

## Quality Suggestions
### Issue: Add tests for missing CLI tool errors and stderr handling
- **Severity**: Low
- **Category**: Completeness
- **Description**: The acceptance criteria specify a clear “command not found” error; tests should assert this and that stderr is surfaced on non-zero exits.
- **Recommendation**: Add unit/integration tests in `src/orchestrator/mod.rs` or `src/orchestrator/script_runner.rs` to validate error strings and stderr capture.

### Issue: Verify env var wiring across all tools
- **Severity**: Low
- **Category**: Completeness
- **Description**: Prompt tests should also validate correct env var setup for `GEMINI_SYSTEM_MD` and `CODEX_INSTRUCTIONS_FILE`.
- **Recommendation**: Extend prompt/runner tests to assert env var injection per tool.

## Verdict
- [ ] APPROVED - Ready for implementation
- [x] NEEDS_REVISION - Address issues above (HIGH)
- [ ] REJECTED - Fundamental problems, needs rethinking

**Next Steps**: Resolve HIGH consistency gaps (spec/task alignment, env setup scope, review tool list) and re-issue updated proposal/tasks before implementation.
