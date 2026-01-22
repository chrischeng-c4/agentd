# Specification: MCP Specification Tooling

## Overview

This specification addresses improvements to the Model Context Protocol (MCP) tools within the Agentd workflow, specifically focusing on the `create_review` tool for implementation feedback and enhancements to the `create_tasks` tool for consistent metadata generation. These changes ensure a more robust and structured interaction between the orchestrator and the implementation/planning agents.

## Requirements

### MS-R1: `create_review` MCP Tool
The system MUST provide a `create_review` MCP tool that implementation agents can use to submit structured findings. This tool MUST generate a `REVIEW.md` file in the change directory.

### MS-R2: `create_tasks` Metadata Generation
The `create_tasks` MCP tool MUST be updated to explicitly include both `id` and `change_id` in the YAML frontmatter of the generated `tasks.md` file to ensure backward compatibility and tool robustness.

### MS-R3: Sequential Workflow Enforcement
The `proposal_engine` MUST be updated to ensure that the transition between proposal generation, spec generation, and task generation is sequential and that each phase's output is validated before proceeding.

### MS-R4: Validation of Tool Inputs
All MCP tools (including `create_spec` and `create_tasks`) MUST validate their input parameters (e.g., minimum lengths, required fields, format of IDs) before performing any file operations.

## Data Model

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["change_id", "verdict", "summary"],
  "properties": {
    "change_id": { "type": "string" },
    "verdict": { "type": "string", "enum": ["APPROVED", "NEEDS_FIX", "MAJOR_ISSUES"] },
    "summary": { "type": "string" },
    "issues": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["title", "severity", "description"],
        "properties": {
          "title": { "type": "string" },
          "severity": { "type": "string", "enum": ["High", "Medium", "Low"] },
          "description": { "type": "string" }
        }
      }
    }
  }
}
```

## Interfaces

```
FUNCTION create_review(change_id: String, verdict: String, summary: String, issues: Array) -> Result<String>
  INPUT: Structured review findings
  OUTPUT: Success message or Error
  SIDE_EFFECTS: Writes agentd/changes/{change_id}/REVIEW.md

FUNCTION create_tasks(change_id: String, tasks: Array) -> Result<String>
  INPUT: List of implementation tasks
  OUTPUT: Success message or Error
  SIDE_EFFECTS: Writes agentd/changes/{change_id}/tasks.md with frontmatter
```

## Acceptance Criteria

### Scenario: Structured Review Submission
- **WHEN** Codex calls `create_review` with structured issues
- **THEN** a `REVIEW.md` file is generated with a "Verdict" header and a list of issues categorized by severity.

### Scenario: Tasks Metadata Consistency
- **WHEN** `agentd plan` generates `tasks.md`
- **THEN** the resulting file contains `change_id: fix-workflow-bugs` in its YAML frontmatter.

### Scenario: Invalid Tool Input
- **WHEN** an agent calls `create_spec` with an overview shorter than 50 characters
- **THEN** the tool returns a validation error and does not create the spec file.
