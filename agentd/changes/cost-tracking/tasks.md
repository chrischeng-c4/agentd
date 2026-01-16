# Tasks

## 1. Data Layer

- [ ] 1.1 Add pricing fields to model configurations
  - File: `src/models/change.rs` (MODIFY)
  - Spec: `specs/usage.md#r1-pricing-configuration`
  - Do: Add `cost_per_1m_input` and `cost_per_1m_output` to `GeminiModelConfig`, `CodexModelConfig`, and `ClaudeModelConfig`. Update default values.
  - Depends: none

- [ ] 1.2 Refactor Telemetry and LlmCall models
  - File: `src/models/frontmatter.rs` (MODIFY)
  - Spec: `specs/usage.md#telemetry-in-state-yaml`
  - Do: Update `LlmCall` to include `step`, `cost_usd` and `timestamp`. Change `Telemetry` to use a `Vec<LlmCall>` called `calls` and add summary fields. Implement custom deserialization for backward compatibility if possible, or accept breaking change.
  - Depends: none

- [ ] 1.3 Update State JSON schema
  - File: `agentd/schemas/state.schema.json` (MODIFY)
  - Spec: `specs/usage.md#telemetry-in-state-yaml`
  - Do: Update the `telemetry` definition to match the new structure, including `step` in required fields.
  - Depends: 1.2

- [ ] 1.4 Update default config
  - File: `agentd/config.toml` (MODIFY)
  - Spec: `specs/usage.md#r1-pricing-configuration`
  - Do: Update the default configuration file to include pricing fields for all models.
  - Depends: 1.1

## 2. Logic Layer

- [ ] 2.1 Implement Usage Metrics parser
  - File: `src/orchestrator/script_runner.rs` (MODIFY)
  - Spec: `specs/usage.md#usage-parser`
  - Do: Add a private method to parse token usage from the captured stdout of CLI tools. Support Gemini/Claude JSON streams and Codex JSON output. Add unit tests for parser.
  - Depends: none

- [ ] 2.2 Update ScriptRunner to return usage
  - File: `src/orchestrator/script_runner.rs` (MODIFY)
  - Spec: `specs/usage.md#usage-parser`
  - Do: Modify `run_llm` to return a `Result<(String, UsageMetrics)>` instead of just `Result<String>`.
  - Depends: 2.1

- [ ] 2.3 Update StateManager recording logic
  - File: `src/state/manager.rs` (MODIFY)
  - Spec: `specs/usage.md#telemetry-recording`
  - Do: Update `record_llm_call` to accept `pricing` info (or `AgentdConfig`), handle the new `Telemetry` structure, calculate cost, and update aggregate totals.
  - Depends: 1.1, 1.2

## 3. Integration

- [ ] 3.1 Update Orchestrators to handle usage metrics
  - File: `src/orchestrator/gemini.rs` (MODIFY)
  - File: `src/orchestrator/claude.rs` (MODIFY)
  - File: `src/orchestrator/codex.rs` (MODIFY)
  - Spec: `specs/usage.md#r3-universal-tracking`
  - Do: Update all orchestrator methods to receive usage from `ScriptRunner` and pass it back to the caller.
  - Depends: 2.2

- [ ] 3.2 Wire up all orchestrator calls to StateManager
  - File: `src/cli/proposal.rs` (MODIFY)
  - File: `src/cli/reproposal.rs` (MODIFY)
  - File: `src/cli/challenge_proposal.rs` (MODIFY)
  - File: `src/cli/implement.rs` (MODIFY)
  - File: `src/cli/fillback.rs` (MODIFY)
  - File: `src/cli/review.rs` (MODIFY)
  - File: `src/cli/resolve_reviews.rs` (MODIFY)
  - File: `src/cli/archive.rs` (MODIFY)
  - Spec: `specs/usage.md#r3-universal-tracking`
  - Do: Ensure every LLM call across all CLI commands (including review, resolve, archive steps) records its telemetry via `StateManager`, passing necessary pricing config.
  - Depends: 2.3, 3.1

## 4. UI & Testing

- [ ] 4.1 Enhance status command with usage summary
  - File: `src/cli/status.rs` (MODIFY)
  - Spec: `specs/usage.md#r6-usage-reporting`
  - Do: Add a "Usage & Cost" section to the status output showing total tokens and cost (4 decimal places).
  - Depends: 2.3

- [ ] 4.2 Add integration test for cost calculation
  - File: `src/state/manager.rs` (MODIFY)
  - Verify: `specs/usage.md#acceptance-criteria`
  - Do: Add unit tests to `StateManager` to verify cost calculation logic with different pricing scenarios.
  - Depends: 2.3
