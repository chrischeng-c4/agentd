# CLAUDE.md

<!-- agentd:start -->
## Agentd: Spec-Driven Development

**IMPORTANT**: Do NOT make direct code changes. Use the SDD workflow below.

| Skill | Purpose |
|-------|---------|
| `/agentd:plan-change` | Planning workflow (proposal → challenge → auto-reproposal loop) |
| `/agentd:impl-change` | Implementation workflow (implement → review → auto-resolve loop) |
| `/agentd:merge-change` | Archive completed change |

All workflows are **state-aware** and resume automatically from the current phase.

Start with: `/agentd:plan-change <id> "<description>"`

### Knowledge Base

System documentation is in `agentd/knowledge/`. Use MCP tools to read:
- `mcp__agentd__list_knowledge` - List all knowledge files
- `mcp__agentd__read_knowledge` - Read specific file

### MCP Tools (Preferred)

**Use MCP tools for all agentd operations.** MCP provides structured input/output that is optimized for LLM interaction.

**Read Operations**:
- `mcp__agentd__list_knowledge` / `mcp__agentd__read_knowledge` - Knowledge base
- `mcp__agentd__list_specs` / `mcp__agentd__read_file` - Specs and proposals
- `mcp__agentd__read_all_requirements` - All requirements for implementation
- `mcp__agentd__list_changed_files` - Git changes for a change

**Creation Operations**:
- `mcp__agentd__create_proposal` - Create proposal.md
- `mcp__agentd__create_spec` - Create spec files
- `mcp__agentd__create_tasks` - Create tasks.md
- `mcp__agentd__append_review` - Add review to proposal
- `mcp__agentd__create_clarifications` - Create clarifications.md

**Diagram Generation**:
- `mcp__agentd__generate_mermaid_flowchart` / `sequence` / `class` / `state` / `erd` / `mindmap`

### CLI Commands (Fallback)

CLI commands are available when MCP is not configured. See `agentd/specs/cli-guide/` for documentation.
<!-- agentd:end -->

# Project Context

## Overview
Agentd is a spec-driven development tool that helps manage and automate the process of creating, reviewing, and implementing changes to a codebase.

## Tech Stack
- Language: Rust
- Framework: None
- Key libraries: clap, serde, toml, anyhow, chrono, git2, mermaid

## Conventions
- Error handling: Using `anyhow` for error propagation.
- Naming: snake_case
- Testing: Using Rust's built-in testing framework (`#[cfg(test)] mod tests`)

## Key Patterns
- Using TOML files for configuration and specifications.
- Structuring changes as proposals, challenges, and refinements.
- Using Mermaid diagrams for visualizing flows and states.

## Architecture
- `src/cli`: Command-line interface definitions and subcommands.
- `src/models`: Data structures for specifications, challenges, and reviews.
- `src/validator`: Validation logic for specifications and challenges.
