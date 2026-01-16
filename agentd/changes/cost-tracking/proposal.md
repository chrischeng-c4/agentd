# Change: cost-tracking

## Summary

Add comprehensive cost and token usage tracking across all LLM orchestrator calls, including pricing configuration and telemetry persistence in `STATE.yaml`.

## Why

Currently, `agentd` only tracks token usage for a few specific steps (proposal, challenge, reproposal) and does not calculate actual costs. To provide better visibility into LLM expenditures and optimize model selection, we need to track every API call, associate it with pricing data, and report cumulative costs to the user.

## What Changes

### Data Models
- **Pricing Configuration**: Add `cost_per_1m_input` and `cost_per_1m_output` fields to `GeminiModelConfig`, `CodexModelConfig`, and `ClaudeModelConfig`.
- **Telemetry Schema**: Refactor `Telemetry` in `STATE.yaml` to support a collection of all LLM calls (including step name) instead of a fixed set of three.
- **LlmCall Enhancements**: Add `step`, `cost_usd`, and `timestamp` to the `LlmCall` model.

### Orchestration Layer
- **Usage Parsing**: Implement a robust parser in `ScriptRunner` to extract token usage from the JSON streams of Gemini, Claude, and Codex CLI tools.
- **Metrics Return**: Update `ScriptRunner::run_llm` and all orchestrator methods to return usage metrics alongside the response text.

### State Management
- **Centralized Tracking**: Update `StateManager::record_llm_call` to calculate costs using the configured pricing and append to the telemetry history.
- **Aggregation**: Add methods to calculate total cost and token consumption for a change.

### Universal Integration
- **Command Coverage**: Wire up all CLI commands (Proposal, Challenge, Reproposal, Implement, Review, Resolve, Archive, etc.) to report usage to `StateManager`.

### User Interface
- **Status Reporting**: Update `agentd status` to display total cost (4 decimals) and a breakdown of token usage.

## Impact

- Affected specs: `specs/usage.md` (new)
- Affected code:
  - `src/models/frontmatter.rs`: `Telemetry` and `LlmCall` structures
  - `src/models/change.rs`: Model configuration structures
  - `src/state/manager.rs`: `record_llm_call` logic
  - `src/orchestrator/script_runner.rs`: CLI output parsing
  - `src/orchestrator/mod.rs`: Orchestrator traits/interfaces
  - `src/cli/*.rs`: All CLI command handlers
- Breaking changes: Yes, `STATE.yaml` telemetry format will be migrated to a list-based structure. Existing active changes may lose previous telemetry data or require manual reset of the `telemetry` block.
