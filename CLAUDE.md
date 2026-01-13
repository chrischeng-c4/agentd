# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Agentd is a Rust-powered spec-driven development orchestrator that installs **Claude Code Skills** for AI-assisted iterative proposal refinement. It orchestrates multiple AI tools (Gemini, Codex, Claude) through a workflow of proposal generation, challenge/review, and implementation.

**Core Workflow**: proposal → challenge → reproposal → implement → verify → archive

## Build & Development Commands

### Building
```bash
# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Run without installing
./target/release/agentd --version
```

### Installation
```bash
# Install from source (puts binary in ~/.cargo/bin/)
cargo install --path .

# Verify installation
agentd --version
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Check for compilation errors (faster than build)
cargo check

# Run clippy linter
cargo clippy

# Auto-fix clippy warnings
cargo clippy --fix

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Running Commands Locally
```bash
# After building, test commands directly
./target/release/agentd init
./target/release/agentd proposal test-change "Test description"
./target/release/agentd list
```

## Architecture

### Module Structure

- **`src/main.rs`**: CLI entry point with clap command parsing
- **`src/lib.rs`**: Public API re-exports
- **`src/models/`**: Data structures
  - `change.rs`: Core `Change` type, `ChangePhase` enum, `AgentdConfig`
  - `requirement.rs`: Requirements and deltas
  - `scenario.rs`: Test scenarios
  - `challenge.rs`: Challenge issues and severity
  - `verification.rs`: Test results and status
- **`src/cli/`**: Command implementations (proposal, challenge, reproposal, implement, verify, archive, etc.)
- **`src/orchestrator/`**: AI tool integration
  - `script_runner.rs`: Executes AI integration scripts
  - `gemini.rs`, `codex.rs`, `claude.rs`: Tool-specific orchestrators
- **`src/parser/`**: Markdown and structured data parsing
- **`src/validator/`**: Format, consistency, and challenge validation
- **`src/ui/`**: Terminal UI (progress bars, tables, colors)

### Key Concepts

**Change**: Central unit of work with phases (Proposed → Challenged → Implementing → Testing → Complete → Archived). Each change has:
- `proposal.md`: Why, what, impact
- `tasks.md`: Implementation checklist
- `diagrams.md`: Mermaid diagrams
- `specs/`: Specification deltas
- `CHALLENGE.md`: Code review feedback (Codex)
- `IMPLEMENTATION.md`: Implementation notes (Claude)
- `VERIFICATION.md`: Test results (Codex)

**AgentdConfig**: Project configuration in `agentd/config.toml`:
- AI CLI commands (gemini, codex, claude)
- Scripts directory path
- Project metadata

**ScriptRunner**: Executes shell scripts in `agentd/scripts/` that integrate with AI tools. Scripts receive change_id and other args, return structured output.

### Directory Layout
```
agentd/                # Main Agentd directory (visible)
  config.toml           # Configuration
  specs/                # Main specifications
    auth/spec.md
    api/spec.md
  changes/              # Active change proposals
    add-oauth/
      proposal.md
      tasks.md
      diagrams.md
      specs/
      CHALLENGE.md
      IMPLEMENTATION.md
      VERIFICATION.md
  archive/              # Completed changes
  scripts/              # AI integration scripts
    gemini-proposal.sh
    gemini-reproposal.sh
    codex-challenge.sh
    codex-verify.sh
    claude-implement.sh

.gemini/                # Gemini Commands (project-specific, hidden)
  commands/agentd/
    proposal.toml       # Proposal generation prompt
    reproposal.toml     # Reproposal refinement prompt
  settings.json         # Tools and auto-approvals

~/.codex/               # Codex Prompts (user-space, global)
  prompts/
    agentd-challenge.md  # Code review prompt
    agentd-verify.md     # Test generation prompt

.claude/skills/         # Claude Code Skills (installed by init, hidden)
  agentd-proposal/
  agentd-challenge/
  agentd-reproposal/
  agentd-implement/
  agentd-verify/
  agentd-archive/
```

## AI CLI Commands Integration

Agentd provides **pre-defined AI commands** for Gemini and Codex, enabling direct CLI usage and simplified scripts.

### Gemini Commands (`.gemini/commands/agentd/`)
Project-specific commands defined in TOML format:
- `proposal.toml` - Generate proposal with 2M context, explore codebase, create spec files
- `reproposal.toml` - Refine proposal based on CHALLENGE.md feedback

### Codex Prompts (`~/.codex/prompts/`)
User-space prompts defined in Markdown format:
- `agentd-challenge.md` - Analyze proposal, identify conflicts and issues
- `agentd-verify.md` - Generate tests from specs, run verification

### Settings
- `.gemini/settings.json` - Allowed tools (write_file, read_file, etc.) and auto-approvals

### Usage Patterns

```bash
# Direct CLI usage (independent of Agentd)
gemini agentd:proposal test-change "Add new feature"    # project-specific
codex agentd-challenge test-change                      # user-space

# Via Agentd CLI (calls the scripts)
agentd proposal test-change "Add new feature"
agentd challenge test-change

# Via Claude Code Skills (orchestrated workflow)
/agentd:proposal test-change "Add new feature"
/agentd:challenge test-change
```

### Architecture Flow

**Gemini (project-specific)**:
1. **Claude Code Skill** → calls Agentd CLI
2. **Agentd CLI** → executes `agentd/scripts/gemini-proposal.sh`
3. **Script** → calls `gemini agentd:proposal`
4. **Gemini CLI** → reads `.gemini/commands/agentd/proposal.toml` and executes

**Codex (user-space)**:
1. **Claude Code Skill** → calls Agentd CLI
2. **Agentd CLI** → executes `agentd/scripts/codex-challenge.sh`
3. **Script** → calls `codex agentd-challenge`
4. **Codex CLI** → reads `~/.codex/prompts/agentd-challenge.md` and executes

## Claude Code Skills Integration

Agentd **installs Skills into Claude Code** via `agentd init`. Skills are markdown files in `.claude/skills/agentd-*/SKILL.md` that define prompts for Claude Code to execute workflow steps.

When users run `/agentd:proposal` in Claude Code, the skill's prompt instructs Claude to:
1. Call `agentd proposal <id> "<description>"`
2. Parse the output
3. Display results to the user

This allows the entire workflow to run within Claude Code's interactive session without bash switching.

## Common Patterns

### Error Handling
Uses `anyhow::Result<T>` throughout for flexible error handling with context:
```rust
use anyhow::{Context, Result};

fn load_file() -> Result<String> {
    std::fs::read_to_string(path)
        .context("Failed to read file")?
}
```

### Async Execution
Uses `tokio` for async operations (script execution, process spawning):
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // async code
}
```

### CLI with Clap
Uses `clap` derive macros for command-line parsing:
```rust
#[derive(Parser)]
#[command(name = "agentd")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

### Terminal UI
Uses `colored`, `indicatif`, `crossterm` for rich terminal output. Progress bars shown during long-running AI script execution.

## Dependencies

Key crates:
- **CLI**: `clap`, `colored`, `dialoguer`, `indicatif`, `crossterm`
- **Parsing**: `pulldown-cmark`, `regex`
- **Serialization**: `serde`, `serde_json`, `toml`, `chrono`
- **Error handling**: `anyhow`, `thiserror`
- **File ops**: `walkdir`, `glob`
- **Git**: `git2`
- **Async**: `tokio` (process execution)

