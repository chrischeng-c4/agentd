# Specification: Cost and Token Tracking

## Overview

This specification defines how `agentd` tracks, calculates, and reports LLM usage metrics. It covers the data structures for pricing, the mechanism for parsing usage from CLI tool outputs, and the persistence of these metrics in the change state.

## Requirements

### R1: Pricing Configuration
The system must allow users to configure input and output token costs per model in `agentd/config.toml`. Costs are defined as USD per 1 million tokens.

### R2: Automated Token Extraction
The `ScriptRunner` must automatically parse token usage (input/output) from the stdout of orchestrated CLI tools (`gemini`, `claude`, `codex`).

### R3: Universal Tracking
Every LLM call made by any orchestrator (Proposal, Challenge, Reproposal, Implementation, Review, Resolve, Archive, Merge, Fillback) must be recorded in the telemetry.

### R4: Cost Calculation
The system must calculate the USD cost of each call based on the model used and the configured pricing at the time of the call.

### R5: State Persistence
All LLM call metrics, including model name, tokens, cost, and duration, must be persisted in the `telemetry` section of `STATE.yaml`.

### R6: Usage Reporting
The `status` command must provide a clear summary of total tokens used and total cost incurred for the active change.

## Data Model

### Pricing in Config
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "cost_per_1m_input": { "type": "number", "minimum": 0 },
    "cost_per_1m_output": { "type": "number", "minimum": 0 }
  }
}
```

### Telemetry in STATE.yaml
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "calls": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["step", "model", "tokens_in", "tokens_out", "cost_usd"],
        "properties": {
          "step": { "type": "string" },
          "model": { "type": "string" },
          "tokens_in": { "type": "integer" },
          "tokens_out": { "type": "integer" },
          "cost_usd": { "type": "number" },
          "duration_ms": { "type": "integer" },
          "timestamp": { "type": "string", "format": "date-time" }
        }
      }
    },
    "total_cost_usd": { "type": "number" }
  }
}
```

## Interfaces

### Usage Parser
```
FUNCTION parse_usage_from_output(output: String, provider: LlmProvider) -> Result<UsageMetrics, Error>
  INPUT: Raw stdout from CLI tool
  OUTPUT: Struct containing input_tokens and output_tokens
  ERRORS: ParseError if usage information is missing or malformed
```

### Telemetry Recording
```
FUNCTION record_llm_call(step: String, model: String, tokens: UsageMetrics, duration_ms: u64, pricing: PricingConfig) -> void
  INPUT: Call metadata, metrics, and pricing configuration
  SIDE_EFFECTS: Updates STATE.yaml with the new call (including step) and recalculates totals
```

## Acceptance Criteria

### Scenario: Successful Cost Tracking
- **WHEN** a proposal is generated using `gemini-3-flash-preview`
- **THEN** a new entry is added to `telemetry.calls` in `STATE.yaml` with correct token counts, calculated cost based on flash pricing, and the `step` set to "proposal".

### Scenario: Multiple Call Aggregation
- **WHEN** multiple orchestrator calls are made (e.g., proposal then challenge)
- **THEN** `telemetry.total_cost_usd` reflects the sum of all individual call costs.

### Scenario: Missing Pricing Info
- **WHEN** a model is used that has no pricing configured
- **THEN** the cost is recorded as 0.0, but token counts are still accurately tracked.

### Scenario: Status Display
- **WHEN** `agentd status` is executed
- **THEN** it displays "Total Cost: $X.XXXX" (4 decimal places) and "Tokens: In: Y, Out: Z".

### Scenario: Universal Coverage
- **WHEN** an archive review is run
- **THEN** a telemetry entry is recorded with `step` set to "archive-review".