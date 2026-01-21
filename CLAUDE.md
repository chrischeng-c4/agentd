# CLAUDE.md

<!-- agentd:start -->
## Agentd: Spec-Driven Development

**IMPORTANT**: Do NOT make direct code changes. Use the SDD workflow below.

| Skill | Purpose |
|-------|---------|
| `/agentd:plan` | Planning workflow (proposal → challenge → auto-reproposal loop) |
| `/agentd:impl` | Implementation workflow (implement → review → auto-resolve loop) |
| `/agentd:archive` | Archive completed change |

All workflows are **state-aware** and resume automatically from the current phase.

Start with: `/agentd:plan <id> "<description>"`

### Knowledge Base

System documentation is in `agentd/knowledge/`. Use CLI commands to read:
```bash
agentd knowledge list                # List all knowledge files
agentd knowledge read <path>         # Read specific file
```

### CLI Commands

The agentd workflows use CLI commands with JSON files for all operations. All commands use the same service layer as the MCP server (when available), ensuring consistent behavior.

**Complete Documentation**:
- **CLI Guide**: See `agentd/specs/cli-guide/README.md`
- **JSON Examples**: See `agentd/specs/cli-guide/examples/`

**Quick Reference**:

**Read Operations** (no JSON needed):
```bash
agentd knowledge list                    # List knowledge files
agentd knowledge read <path>             # Read knowledge file
agentd spec list <change-id>             # List specs
agentd file read <change-id> proposal    # Read proposal.md
agentd implementation read-all <id>      # Read all requirements
agentd implementation list-files <id>    # List changed files
```

**Creation Operations** (using JSON files):
```bash
agentd proposal create <id> --json-file proposal.json
agentd spec create <id> <spec-id> --json-file spec.json
agentd tasks create <id> --json-file tasks.json
agentd proposal review <id> --json-file review.json
agentd clarifications create <id> --json-file clarifications.json
agentd knowledge write <path> --json-file knowledge.json
```

**Phase Summary**:
- **Phase 1 (Read-Only)**: 5 commands - knowledge, spec, file operations
- **Phase 2 (Creation)**: 4 commands - proposal, spec, tasks, review
- **Phase 3 (Specialized)**: 5 commands - implementation, clarifications, knowledge write

**LLM Usage Pattern**:
1. Generate JSON from examples in `agentd/specs/cli-guide/examples/`
2. Write JSON to temporary file
3. Execute CLI command with `--json-file`
4. Parse output to verify success
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