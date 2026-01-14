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