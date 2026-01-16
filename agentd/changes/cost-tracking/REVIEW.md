# Code Review Report: cost-tracking

**Iteration**: 0

## Summary
Core cost-tracking requirements are not met because LLM usage metrics are collected but never recorded into `STATE.yaml`. Tests pass, but missing wiring means status reporting and telemetry persistence do not work in real workflows.

## Test Results
**Overall Status**: PASS

### Test Summary
- Total tests: 181 (180 unit/integration + 1 doc-test)
- Passed: 181
- Failed: 0
- Skipped: 0
- Coverage: Not reported

## Security Scan Results
**Status**: WARNINGS

### cargo audit (Dependency Vulnerabilities)
- Not run (cargo-audit not available)

### semgrep (Code Pattern Scan)
- Not run (semgrep not available)

### Linter Security Rules
- Clippy emitted 670 warnings (mostly style/pedantic; no security-specific failures reported)

## Best Practices Issues
None identified beyond the requirement compliance issues below.

## Requirement Compliance Issues

### Issue: LLM telemetry is never recorded, so R3/R5/R6 fail
- **Severity**: High
- **Category**: Missing Feature
- **Requirement**: `specs/usage.md#r3-universal-tracking`, `specs/usage.md#r5-state-persistence`, `specs/usage.md#r6-usage-reporting`
- **Evidence**: Usage metrics are returned but discarded in CLI workflows (e.g., `src/cli/proposal.rs:190`, `src/cli/implement.rs:120`). There are no calls to `StateManager::record_llm_call` outside tests.
- **Description**: Telemetry never gets appended to `STATE.yaml`, so total cost/tokens and per-step breakdowns are never persisted or shown.
- **Recommendation**: For every orchestrator call, load the change’s `StateManager`, call `record_llm_call` with step name, selected model, usage tokens/duration, and pricing from `AgentdConfig`, then save the state.

### Issue: Missing-pricing costs are not recorded as 0.0
- **Severity**: Medium
- **Category**: Wrong Behavior
- **Requirement**: `specs/usage.md#acceptance-criteria` (Missing Pricing Info)
- **File**: `src/state/manager.rs:419`
- **Description**: `calculate_cost` returns `None` when pricing is missing, so `cost_usd` is omitted and totals are not updated. The spec expects a recorded cost of `0.0`.
- **Recommendation**: Always return `Some(0.0)` when tokens exist but pricing is missing, and keep totals consistent with that behavior.

### Issue: Status output hides total cost when it is 0.0
- **Severity**: Medium
- **Category**: Wrong Behavior
- **Requirement**: `specs/usage.md#r6-usage-reporting`
- **File**: `src/cli/status.rs:73`
- **Description**: The total cost line is only printed when `total_cost_usd > 0.0`, which suppresses the required “Total Cost: $X.XXXX” display when pricing is missing.
- **Recommendation**: Always print the total cost line, even when it is `$0.0000`.

### Issue: Telemetry schema is weaker than the spec
- **Severity**: Medium
- **Category**: Requirement Mismatch
- **Requirement**: `specs/usage.md#telemetry-in-state-yaml`
- **File**: `agentd/schemas/state.schema.json`
- **Description**: The schema only requires `step` for an LLM call, while the spec requires `step`, `model`, `tokens_in`, `tokens_out`, and `cost_usd`.
- **Recommendation**: Align schema required fields with the spec (or update the spec if optional fields are intended).

## Consistency Issues

### Issue: Secondary default config lacks pricing fields
- **Severity**: Medium
- **Category**: Configuration Consistency
- **Location**: `agentd/agentd/config.toml`
- **Description**: This default config template does not include `cost_per_1m_input`/`cost_per_1m_output`, unlike `agentd/config.toml`.
- **Recommendation**: Update the template config to keep defaults consistent.

## Test Quality Issues

### Issue: No tests assert telemetry wiring from CLI workflows
- **Severity**: Medium
- **Category**: Coverage
- **Description**: Tests cover `StateManager` and parsers, but none validate that CLI workflows record telemetry or that status output includes cost data.
- **Recommendation**: Add integration tests that execute a CLI workflow with mocked usage and assert `STATE.yaml` telemetry updates and status output.

## Verdict
- [ ] APPROVED - Ready for merge (all tests pass, no HIGH issues)
- [x] NEEDS_CHANGES - Address issues above (telemetry wiring + spec compliance)
- [ ] MAJOR_ISSUES - Fundamental problems (failing tests or critical security)

**Next Steps**: Wire telemetry recording into CLI orchestrator calls, update cost/telemetry behavior and schema, and add integration tests for telemetry persistence and status output.
