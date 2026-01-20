---
id: mcp-spec-tool-2
type: tasks
version: 1
created_at: 2026-01-19T14:15:00Z
updated_at: 2026-01-19T22:10:00Z
proposal_ref: mcp-spec-tool-2
summary:
  total: 13
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 13
layers:
  data:
    task_count: 4
    estimated_files: 2
  logic:
    task_count: 6
    estimated_files: 10
  integration:
    task_count: 2
    estimated_files: 3
  testing:
    task_count: 1
    estimated_files: 1
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 13 implementation tasks for change `mcp-spec-tool-2`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 4 | 🔲 Pending |
| Logic Layer | 6 | 🔲 Pending |
| Integration Layer | 2 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 1. Data Layer

- [ ] 1.1 Register `append_review` MCP tool
  - File: `src/mcp/tools/mod.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R3`
  - Do: Expose the `append_review` tool in the `ToolRegistry`. Define its schema and wire it to the execution logic.
  - Depends: none

- [ ] 1.2 Implement `append_review` tool execution
  - File: `src/mcp/tools/proposal.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R3`
  - Do: Add a `definition()` for `append_review` and an `execute_append_review` function that wraps the existing `append_review` logic. Include `resolved` in allowed statuses.
  - Depends: 1.1

- [ ] 1.3 Enhance `create_proposal` for metadata preservation
  - File: `src/mcp/tools/proposal.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R2`
  - Do: Update `execute` to read existing `proposal.md` if it exists and preserve fields like `created_at` and `author`.
  - Depends: none

- [ ] 1.4 Add validation logic to MCP tools
  - File: `src/mcp/tools/proposal.rs`, `src/mcp/tools/spec.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R4`
  - Do: Ensure MCP tools validate inputs (min length for overview, valid YAML blocks, correct file paths) before writing files. Return errors for invalid inputs.
  - Depends: none

## 2. Logic Layer

- [ ] 2.1 Update orchestrator prompts for MCP enforcement
  - File: `src/orchestrator/prompts.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R1`, `specs/mcp-tool-enforcement.md#R6`
  - Do: Update ALL prompts to strictly require MCP tool usage. Update `codex_challenge_prompt` to use `append_review` tool and stop referencing `CHALLENGE.md`. Ensure instructions are advisory but firm.
  - Depends: 1.2

- [ ] 2.2 Update guidance templates and skeletons
  - File: `templates/GEMINI.md`, `templates/AGENTS.md`, `templates/skeletons/planning/*.md`, `src/context.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R5`
  - Do: Remove references to `CHALLENGE.md`. **Delete `templates/skeletons/planning/challenge.md`** and remove its registration in `src/context.rs`.
  - Depends: none

- [ ] 2.3 Update `proposal.toml` and `reproposal.toml`
  - File: `templates/gemini/commands/agentd/proposal.toml`, `templates/gemini/commands/agentd/reproposal.toml` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R5`
  - Do: Remove "Option B: Direct File Writing (Fallback)" and ensure instructions point exclusively to MCP tools. Remove references to `CHALLENGE.md`.
  - Depends: none

- [ ] 2.4 Refactor Challenge parsing and validation
  - File: `src/parser/challenge.rs`, `src/cli/validate_challenge.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R6`
  - Do: Exclusively use `<review>` blocks in `proposal.md` for verdict parsing and validation. Remove logic that searches for `CHALLENGE.md`.
  - Depends: none

- [ ] 2.5 Remove CHALLENGE.md from state and models
  - File: `src/state/manager.rs`, `src/models/change.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R6`
  - Do: Remove all references to `CHALLENGE.md` from state tracking and change model paths. Update state transitions to not require CHALLENGE.md.
  - Depends: none

- [ ] 2.6 Remove CHALLENGE.md from UI viewer and schema validation
  - File: `src/ui/viewer/manager.rs`, `src/validator/schema.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R6`
  - Do: Remove `CHALLENGE.md` from viewer allowlist and schema selection. Delete `agentd/schemas/challenge.schema.json`.
  - Depends: none

## 3. Integration Layer

- [ ] 3.1 Refactor `ProposalEngine` for consolidated artifacts
  - File: `src/cli/proposal_engine.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R2`, `specs/mcp-tool-enforcement.md#R6`
  - Do: Stop generating `CHALLENGE.md` skeleton. Update re-challenge and reproposal loops to rely solely on `proposal.md` reviews.
  - Depends: 2.1, 2.4, 2.5, 2.6

- [ ] 3.2 Migration for existing changes with CHALLENGE.md
  - File: `src/cli/proposal_engine.rs` (MODIFY)
  - Spec: `specs/mcp-tool-enforcement.md#R6`
  - Do: Add graceful handling when `CHALLENGE.md` exists in old changes. Either auto-convert to review blocks in proposal.md or skip with warning.
  - Depends: 3.1

## 4. Testing Layer

- [ ] 4.1 Create E2E test for MCP enforcement and artifact consolidation
  - File: `tests/mcp_enforcement_test.rs` (CREATE)
  - Verify: `specs/mcp-tool-enforcement.md#Acceptance Criteria`
  - Do: Mock Gemini and Codex output with MCP tool calls. Verify `proposal.md` contains both the proposal and the review block, and that `CHALLENGE.md` is NOT created.
  - Depends: 3.1

</tasks>
