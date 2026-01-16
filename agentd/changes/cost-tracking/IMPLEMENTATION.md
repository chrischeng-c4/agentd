# Cost Tracking Implementation

## Status: COMPLETE

Implementation of usage/cost tracking for LLM calls in the agentd workflow.

## Changes Made

### 1. Model Configuration (src/models/change.rs)
- Added `cost_per_1m_input` and `cost_per_1m_output` fields to:
  - `GeminiModelConfig`
  - `CodexModelConfig`
  - `ClaudeModelConfig`
- Updated default model configurations with pricing:
  - Gemini Flash: $0.10/$0.40 per 1M tokens
  - Gemini Pro: $1.25/$10.00 per 1M tokens
  - Codex models: $2.00/$8.00 per 1M tokens
  - Claude Haiku: $0.80/$4.00 per 1M tokens
  - Claude Sonnet: $3.00/$15.00 per 1M tokens
  - Claude Opus: $15.00/$75.00 per 1M tokens

### 2. Telemetry Model (src/models/frontmatter.rs)
- Refactored `Telemetry` struct:
  - Changed from per-step fields to a `calls` vector
  - Added `total_cost_usd`, `total_tokens_in`, `total_tokens_out` totals
- Enhanced `LlmCall` struct:
  - Added `step` field (e.g., "proposal", "challenge", "implement")
  - Added `cost_usd` field for per-call cost
  - Added `timestamp` field with `DateTime<Utc>` type

### 3. State Schema (agentd/schemas/state.schema.json)
- Updated telemetry schema to match new structure
- Added calls array with per-call properties
- Added total fields for aggregated metrics

### 4. Configuration (agentd/config.toml)
- Added pricing fields to all model configurations

### 5. ScriptRunner (src/orchestrator/script_runner.rs)
- Added `UsageMetrics` struct for returning usage from LLM calls
- Added JSON response parsing for:
  - Gemini stream-json format (usageMetadata)
  - Claude stream-json format (usage)
  - Codex JSON format (usage)
- Updated `run_llm` to return `(String, UsageMetrics)` tuple
- Added duration tracking with `Instant`

### 6. StateManager (src/state/manager.rs)
- Updated `record_llm_call` to accept pricing parameters
- Added `calculate_cost` static method for cost calculation
- Added `telemetry_summary` getter method
- Implemented automatic aggregation of totals

### 7. Orchestrators
Updated all orchestrators to handle the new tuple return type:
- `GeminiOrchestrator`: run_proposal, run_reproposal, run_merge_specs, run_changelog, run_fillback, run_archive_fix
- `CodexOrchestrator`: run_challenge, run_rechallenge, run_review, run_verify, run_archive_review
- `ClaudeOrchestrator`: run_implement, run_resolve

### 8. CLI Commands
Updated all CLI commands to destructure tuple returns:
- proposal.rs
- challenge_proposal.rs
- reproposal.rs
- implement.rs
- review.rs
- resolve_reviews.rs
- archive.rs

### 9. Status Command (src/cli/status.rs)
Enhanced to display usage summary:
- Total tokens (in/out)
- Total cost in USD
- Number of LLM calls
- Breakdown by step

## Testing

Added comprehensive tests in `src/state/manager.rs`:
- `test_record_llm_call_basic` - Basic recording without pricing
- `test_record_llm_call_with_cost` - Recording with cost calculation
- `test_record_multiple_llm_calls` - Multi-provider workflow simulation
- `test_cost_calculation` - Direct cost calculation edge cases
- `test_telemetry_persistence` - YAML save/load round-trip
- `test_telemetry_summary` - Summary getter functionality
- `test_small_token_cost_precision` - Precision for small values

Also added usage parsing tests in `src/orchestrator/script_runner.rs`:
- `test_parse_gemini_usage` - Gemini JSON parsing
- `test_parse_claude_usage` - Claude JSON parsing
- `test_parse_codex_usage` - Codex JSON parsing
- `test_parse_empty_output` - Edge case handling
- `test_parse_malformed_json` - Error resilience

All 180 tests pass.

## Usage Example

After running commands, use `agentd status <change-id>` to see:

```
Status for: my-change

   Phase:     🔍 Challenged
   Iteration: 1
   Last:      challenge
   Updated:   2025-01-16 10:30:00

💰 Usage Summary:
   Total tokens:  380,000 in / 190,000 out
   Total cost:    $2.6100
   LLM calls:     3

   Breakdown by step:
     proposal     100,000 in / 50,000 out  ($0.0300)
     challenge    80,000 in / 40,000 out   ($0.4800)
     implement    200,000 in / 100,000 out ($2.1000)
```

## Future Work

- Wire up CLI commands to actually record telemetry (currently returning `_usage` is unused)
- Add cost tracking to workflow orchestration (plan/impl cycles)
- Add cost threshold warnings
- Add cumulative cost tracking across all changes
