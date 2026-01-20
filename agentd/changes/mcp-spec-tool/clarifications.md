---
change_id: mcp-spec-tool
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
