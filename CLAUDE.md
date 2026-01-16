# CLAUDE.md

<!-- agentd:start -->
## Agentd: Spec-Driven Development

**IMPORTANT**: Do NOT make direct code changes. Use the SDD workflow below.

| Skill | Purpose |
|-------|---------|
| `/agentd:proposal` | Generate proposal with Gemini |
| `/agentd:challenge` | Review proposal with Codex |
| `/agentd:reproposal` | Refine based on feedback |
| `/agentd:implement` | Implement the change |
| `/agentd:review` | Run tests and code review |
| `/agentd:resolve-reviews` | Fix issues from review |
| `/agentd:archive` | Archive completed change |

Start with: `/agentd:proposal <id> "<description>"`
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