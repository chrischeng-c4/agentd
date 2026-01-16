# Code Review Report: move-shell-to-rust

**Iteration**: 0

## Summary
Tests pass, but there are multiple requirement-compliance gaps that block approval. Security tooling reports an unmaintained dependency warning, semgrep output is empty, and clippy reports a large number of warnings.

## Test Results
**Overall Status**: PASS

### Test Summary
- Total tests: 125
- Passed: 125
- Failed: 0
- Skipped: 0
- Coverage: Not reported

### Failed Tests (if any)
- None

## Security Scan Results
**Status**: WARNINGS

### cargo audit (Dependency Vulnerabilities)
- Warning: `number_prefix` 0.4.0 is unmaintained (RUSTSEC-2025-0119), via `indicatif` -> `agentd`.

### semgrep (Code Pattern Scan)
- No output provided (unable to confirm whether the scan ran or if there were zero findings).

### Linter Security Rules
- Clippy reported 642 warnings under `-W clippy::pedantic` (style and lint warnings).

## Best Practices Issues
[HIGH priority - must fix]

- None identified beyond requirement compliance issues below.

## Requirement Compliance Issues
[HIGH priority - must fix]

### Issue: ModelSelector complexity never varies from Medium
- **Severity**: High
- **Category**: Wrong Behavior
- **Requirement**: specs/orchestrator.md#R4
- **Description**: All orchestrator calls use `Complexity::Medium`, so the ModelSelector never adapts based on change complexity or assessment. This defeats dynamic model selection required by R4.
- **Recommendation**: Use `Change::assess_complexity` or stored complexity from change metadata when calling orchestrators, and propagate it through CLI entry points.
- **File**: src/cli/proposal.rs:186
- **File**: src/cli/implement.rs:106
- **File**: src/cli/review.rs:44

### Issue: Legacy shell scripts still present
- **Severity**: High
- **Category**: Missing Feature
- **Requirement**: tasks.md#3.4, proposal.md "Clean up"
- **Description**: `agentd/scripts/*.sh` files remain in the repository even though the change requires removing legacy scripts after moving orchestration into Rust.
- **Recommendation**: Remove the legacy scripts or move them to an archived location that is clearly excluded from runtime usage.
- **File**: agentd/scripts/gemini-proposal.sh

### Issue: CLI output is not streamed in real time
- **Severity**: High
- **Category**: Wrong Behavior
- **Requirement**: specs/orchestrator.md#R5
- **Description**: `ScriptRunner::run_cli` captures stdout/stderr into strings and only updates the spinner message, but never forwards the command output to the terminal. This breaks the "stream output and update progress UI in real-time" requirement and is a UX regression from the shell pipeline.
- **Recommendation**: Stream stdout/stderr lines to stdout/stderr while still capturing them for return, and keep the spinner updates.
- **File**: src/orchestrator/script_runner.rs:60

## Consistency Issues
[MEDIUM priority - should fix]

- None identified.

## Test Quality Issues
[MEDIUM priority - should fix]

### Issue: Missing integration tests for orchestrator execution pipeline
- **Severity**: Medium
- **Category**: Scenario
- **Description**: tasks.md#4.2 calls for integration tests that mock command execution and validate stderr capture/streaming and tool pre-processing. The current tests are unit-level and do not cover end-to-end orchestration behavior.
- **Recommendation**: Add integration tests that mock `tokio::process::Command` execution or abstract the process runner so outputs can be asserted without external CLI binaries.

## Verdict
- [ ] APPROVED - Ready for merge (all tests pass, no HIGH issues)
- [x] NEEDS_CHANGES - Address HIGH issues (ModelSelector complexity usage, legacy scripts removal, and stdout/stderr streaming)
- [ ] MAJOR_ISSUES - Fundamental problems (failing tests or critical security)

**Next Steps**: Fix the HIGH requirement compliance issues, then add the missing integration tests and re-run security scans to confirm clean status.
