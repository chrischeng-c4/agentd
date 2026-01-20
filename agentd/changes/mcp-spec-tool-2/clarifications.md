---
change_id: mcp-spec-tool-2
created: 2026-01-19
source: "GitHub Issue #6"
---

# Clarifications: MCP Spec Tool Integration

## Q1: Output Strategy
**Question**: How should agents output spec files?

**Answer**: No agent (Gemini, Codex, or any other) should output files directly. All file generation must go through MCP tools.

**Rationale**: This enforces consistent formatting, provides automatic validation, and creates a single point of control for file structure. MCP tools act as the "gatekeeper" for all file operations.

## Q2: Scope
**Question**: Should we update all spec generation paths or just proposal generation?

**Answer**: All spec generation paths.

**Rationale**: Consistency across the entire codebase. Any path that generates specs should use the `create_spec` MCP tool to ensure format compliance.

## Q3: Implementation Approach
**Question**: How should structured data be extracted from agent output?

**Answer**: Agents should output structured data (JSON/TOML) that maps to MCP tool parameters, not raw markdown.

**Rationale**:
- Cleaner separation of concerns (content vs formatting)
- Easier validation of agent output
- MCP tool handles all formatting decisions
- Reduces parsing complexity and edge cases

## Q4: Challenge Artifact Strategy (IMPORTANT)
**Question**: Should Codex write to CHALLENGE.md or use MCP tool to append reviews to proposal.md?

**Answer**: **Remove CHALLENGE.md entirely.** Codex should use the `append_review` MCP tool to write reviews directly to proposal.md.

**Rationale**:
- Single source of truth (proposal.md contains both proposal and reviews)
- Consistent with "all file writes through MCP tools" principle
- Simplifies the workflow by eliminating a separate artifact
- Reviews are naturally tied to the proposal they evaluate

## Q5: Bypass Detection
**Question**: How should the system detect when agents bypass MCP tools?

**Answer**: This should be advisory/prompt-based enforcement only for now. No runtime guard needed in this iteration.

**Rationale**:
- Prompt-level enforcement is sufficient for initial implementation
- Runtime guards add complexity and can be added later if needed
- Focus on the core goal: making MCP tools the standard path
