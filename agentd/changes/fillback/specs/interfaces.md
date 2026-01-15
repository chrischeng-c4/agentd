# Specification: Fillback Interfaces

## Overview
Defines the `ImportStrategy` trait and the specific implementations for handling different import sources.

## Requirements

### R1: ImportStrategy Trait
The system must define a common trait `ImportStrategy` that all import methods implement, ensuring a consistent execution interface.

### R2: OpenSpec Strategy
The `OpenSpecStrategy` must accept YAML or JSON files adhering to the OpenSpec standard and convert them into Agentd Markdown specifications.

### R3: Speckit Strategy
The `SpeckitStrategy` must parse GitHub Speckit markdown files and restructure them into Agentd specifications.

### R4: Code Strategy
The `CodeStrategy` must analyze source code files, respecting `.gitignore` rules, and use the LLM Orchestrator to generate high-level technical designs.

### R5: Spec Generation Request
The system must use a defined data structure `SpecGenerationRequest` when communicating with the Orchestrator to ensure context (file paths, content) is passed correctly.

## Interfaces

### ImportStrategy Trait Definition
```rust
#[async_trait]
trait ImportStrategy {
    // Main entry point for the strategy
    async fn execute(&self, source: &Path, change_id: &str) -> Result<()>;
    
    // Optional: Identify if this strategy applies to the given path (for auto-detection)
    fn can_handle(&self, source: &Path) -> bool;
}
```

## Data Model

### SpecGenerationRequest
Structure sent to the Orchestrator for Code strategy.
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "files": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "path": { "type": "string" },
          "content": { "type": "string" }
        }
      }
    },
    "prompt": { "type": "string" }
  }
}
```

## Acceptance Criteria

### Scenario: Code Strategy GitIgnore
- **WHEN** `CodeStrategy` scans a directory containing a `.gitignore` file
- **THEN** it must exclude files and directories matching the ignore patterns from the analysis.

### Scenario: OpenSpec Format Support
- **WHEN** `OpenSpecStrategy` is executed on a valid JSON OpenSpec file
- **THEN** it successfully parses and generates the corresponding Agentd spec.

### Scenario: Output Location
- **WHEN** any strategy completes execution
- **THEN** all generated artifacts must be present in `agentd/changes/<change_id>/`.