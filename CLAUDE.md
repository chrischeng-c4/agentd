# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Specter is a Rust-powered spec-driven development orchestrator that installs **Claude Code Skills** for AI-assisted iterative proposal refinement. It orchestrates multiple AI tools (Gemini, Codex, Claude) through a workflow of proposal generation, challenge/review, and implementation.

**Core Workflow**: proposal → challenge → reproposal → implement → verify → archive

## Build & Development Commands

### Building
```bash
# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Run without installing
./target/release/specter --version
```

### Installation
```bash
# Install from source (puts binary in ~/.cargo/bin/)
cargo install --path .

# Verify installation
specter --version
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
./target/release/specter init
./target/release/specter proposal test-change "Test description"
./target/release/specter list
```

## Architecture

### Module Structure

- **`src/main.rs`**: CLI entry point with clap command parsing
- **`src/lib.rs`**: Public API re-exports
- **`src/models/`**: Data structures
  - `change.rs`: Core `Change` type, `ChangePhase` enum, `SpecterConfig`
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

**SpecterConfig**: Project configuration in `specter/config.toml`:
- AI CLI commands (gemini, codex, claude)
- Scripts directory path
- Project metadata

**ScriptRunner**: Executes shell scripts in `specter/scripts/` that integrate with AI tools. Scripts receive change_id and other args, return structured output.

### Directory Layout
```
specter/                # Main Specter directory (visible)
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
  commands/specter/
    proposal.toml       # Proposal generation prompt
    reproposal.toml     # Reproposal refinement prompt
  settings.json         # Tools and auto-approvals

~/.codex/               # Codex Prompts (user-space, global)
  prompts/
    specter-challenge.md  # Code review prompt
    specter-verify.md     # Test generation prompt

.claude/skills/         # Claude Code Skills (installed by init, hidden)
  specter-proposal/
  specter-challenge/
  specter-reproposal/
  specter-implement/
  specter-verify/
  specter-archive/
```

## AI CLI Commands Integration

Specter provides **pre-defined AI commands** for Gemini and Codex, enabling direct CLI usage and simplified scripts.

### Gemini Commands (`.gemini/commands/specter/`)
Project-specific commands defined in TOML format:
- `proposal.toml` - Generate proposal with 2M context, explore codebase, create spec files
- `reproposal.toml` - Refine proposal based on CHALLENGE.md feedback

### Codex Prompts (`~/.codex/prompts/`)
User-space prompts defined in Markdown format:
- `specter-challenge.md` - Analyze proposal, identify conflicts and issues
- `specter-verify.md` - Generate tests from specs, run verification

### Settings
- `.gemini/settings.json` - Allowed tools (write_file, read_file, etc.) and auto-approvals

### Usage Patterns

```bash
# Direct CLI usage (independent of Specter)
gemini specter:proposal test-change "Add new feature"    # project-specific
codex specter-challenge test-change                      # user-space

# Via Specter CLI (calls the scripts)
specter proposal test-change "Add new feature"
specter challenge test-change

# Via Claude Code Skills (orchestrated workflow)
/specter:proposal test-change "Add new feature"
/specter:challenge test-change
```

### Architecture Flow

**Gemini (project-specific)**:
1. **Claude Code Skill** → calls Specter CLI
2. **Specter CLI** → executes `specter/scripts/gemini-proposal.sh`
3. **Script** → calls `gemini specter:proposal`
4. **Gemini CLI** → reads `.gemini/commands/specter/proposal.toml` and executes

**Codex (user-space)**:
1. **Claude Code Skill** → calls Specter CLI
2. **Specter CLI** → executes `specter/scripts/codex-challenge.sh`
3. **Script** → calls `codex specter-challenge`
4. **Codex CLI** → reads `~/.codex/prompts/specter-challenge.md` and executes

## Claude Code Skills Integration

Specter **installs Skills into Claude Code** via `specter init`. Skills are markdown files in `.claude/skills/specter-*/SKILL.md` that define prompts for Claude Code to execute workflow steps.

When users run `/specter:proposal` in Claude Code, the skill's prompt instructs Claude to:
1. Call `specter proposal <id> "<description>"`
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
#[command(name = "specter")]
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

