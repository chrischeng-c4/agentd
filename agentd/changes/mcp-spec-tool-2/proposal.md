---
id: mcp-spec-tool-2
type: proposal
version: 1
created_at: 2026-01-19T14:10:00Z
updated_at: 2026-01-19T14:45:00Z
author: mcp
status: proposed
iteration: 4
summary: "Enforce structured output and MCP tool usage for all file generation in Agentd."
impact:
  scope: moderate
  affected_files: 18
  new_files: 1
---

<proposal>

# Change: mcp-spec-tool-2

## Summary
Enforce structured output and MCP tool usage for all file generation and modifications across all agents (Gemini, Codex, etc.). This includes exposing `append_review` as an MCP tool, consolidating challenge artifacts into `proposal.md`, and updating all prompts and templates to forbid direct file editing.

## Why
Direct file editing by agents (using legacy prompts) bypasses validation logic and leads to inconsistent formatting. By forcing all generation through MCP tools (e.g., `create_proposal`, `create_spec`, `create_tasks`, and a new `append_review`), we ensure that:
1. All files follow the strictly defined schemas and XML wrapping conventions.
2. Content is validated before being written (e.g., minimum lengths, required sections, valid YAML blocks).
3. The "Structured Output" pattern is maintained throughout the planning and challenge phases.
4. The system has a single point of entry for all artifact modifications, simplifying auditing and consistency checks.

## What Changes
- **MCP Tools**:
  - Expose the existing `append_review` logic as a formal MCP tool.
  - Register `append_review` in the `ToolRegistry`.
  - **Metadata Preservation**: Update `create_proposal` and other tools to preserve existing frontmatter when overwriting files.
- **Artifact Consolidation**:
  - **Remove CHALLENGE.md entirely**. Review blocks will be appended directly to `proposal.md` using the `append_review` MCP tool.
  - Update all logic that generates, parses, or validates `CHALLENGE.md` to use the `<review>` blocks in `proposal.md` instead.
- **src/orchestrator/prompts.rs**:
  - Update all Gemini planning prompts (`gemini_proposal_prompt`, etc.) to require MCP tools.
  - Update `codex_challenge_prompt` to use `append_review` MCP tool instead of direct file editing and stop mentioning `CHALLENGE.md`.
  - Update all self-review prompts to require MCP tools for any fixes.
- **Templates**:
  - Update `templates/GEMINI.md` and `templates/AGENTS.md` to remove references to `CHALLENGE.md` and prioritize MCP tool usage.
  - Update `templates/gemini/commands/agentd/proposal.toml` and `reproposal.toml` to remove direct-writing fallbacks and references to `CHALLENGE.md`.
- **Skeletons**:
  - **Delete the `challenge.md` skeleton**.
  - Update planning skeletons to include warnings against direct editing and reminders to use tools.
- **src/cli/proposal_engine.rs**:
  - Refactor to stop generating `CHALLENGE.md` and to parse verdicts directly from `proposal.md`.
  - Unify workflows and ensure sequential, tool-based generation is favored.

## Impact
- **Scope**: moderate
- **Affected Files**: ~18
- **New Files**: ~1
- **Affected Specs**:
  - `mcp-tool-enforcement`
- **Affected Code**:
  - `src/mcp/tools/mod.rs`
  - `src/mcp/tools/proposal.rs`
  - `src/mcp/tools/spec.rs`
  - `src/orchestrator/prompts.rs`
  - `src/cli/proposal_engine.rs`
  - `src/context.rs`
  - `src/parser/challenge.rs`
  - `src/cli/validate_challenge.rs`
  - `src/state/manager.rs`
  - `src/models/change.rs`
  - `src/ui/viewer/manager.rs`
  - `src/validator/schema.rs`
  - `templates/GEMINI.md`
  - `templates/AGENTS.md`
  - `templates/skeletons/planning/*.md`
  - `templates/gemini/commands/agentd/proposal.toml`
  - `templates/gemini/commands/agentd/reproposal.toml`
- **Deleted Files**:
  - `agentd/schemas/challenge.schema.json`
  - `templates/skeletons/planning/challenge.md`
- **Breaking Changes**: Yes - `CHALLENGE.md` will no longer be generated. Existing scripts or tools relying on this file must be updated.

</proposal>