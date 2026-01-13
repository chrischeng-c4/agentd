# Specter

**Spec**-driven Development Orches**ter** (Orchestrator)

A Rust-based tool for spec-driven development (SDD) that orchestrates multiple AI models to optimize for cost, quality, and safety.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Software development with Large Language Models (LLMs) often faces a trilemma of **Context**, **Cost**, and **Control**.
- High-context models are expensive for routine tasks.
- Code-specialized models may lack broader architectural awareness.
- Single-model workflows often drift from specifications.

**Specter solves this by orchestrating the right tool for the right job:**
- **Gemini CLI** (configurable, default: Gemini 1.5 Pro with 2M context): Deep codebase exploration and proposal generation. High context, low cost.
- **Codex CLI** (configurable, default: GPT-4 or o1): Rigorous code review, testing, and quality gating. High precision for code analysis.
- **Claude Code** (Claude 3.5 Sonnet): Interactive implementation with best-in-class coding capability.

> **Note**: Model selection is configurable through external CLI tools. The defaults listed above represent recommended configurations for optimal cost/quality balance.

## Why Specter?

Specter is designed for developers who want the velocity of AI coding without sacrificing architectural integrity or incurring massive API costs.

- **💰 Cost Efficiency**: Reduces development costs by offloading context-heavy tasks to more efficient models.
- **🛡️ Safety & Quality**: Introduces automated "Challenge" phases where an AI reviewer critiques the AI proposer *before* any code is written.
- **🔄 Self-Correction**: Automatically detects and fixes hallucinations or conflicts through iterative refinement loops.
- **✅ Rigorous Archiving**: The `archive` workflow is a 7-step quality gate that validates specs, merges changes, and verifies correctness before archiving.

## Cost Analysis

By specializing model usage, Specter can significantly reduce token costs compared to using a single high-end model for the entire workflow.

> **Estimated savings** based on typical feature development in a 100+ file codebase. Actual costs vary based on codebase size, API pricing, and model selection.

| Development Phase | Single-Model Approach | Specter Multi-Model | Estimated Savings |
|-------------------|----------------------|---------------------|-------------------|
| **Proposal & Research** | High Cost (Deep context) | **Lower Cost** (Gemini) | ~70-80% |
| **Code Review** | High Cost | **Lower Cost** (Codex) | ~60-75% |
| **Implementation** | Medium Cost | Medium Cost (Claude) | ~0% |
| **Test Generation** | Medium Cost | **Lower Cost** (Codex) | ~50-60% |

*Note: Session resumption and caching can provide additional 30-40% token savings during reproposal iterations.*

## Key Features

- **Automated Challenge-Reproposal Loop**: Proposal workflow automatically challenges and refines the plan before implementation.
- **Iterative Refinement**: Built-in quality gates ensure plans are solid before code is written.
- **Conflict Resolution**: Automatic detection and resolution of conflicting change-ids.
- **Safety Rollbacks**: Integrated backup and rollback systems during critical phases like archiving.
- **Context Awareness**: Leverages 2M+ token context windows for holistic project understanding.

## Installation

### From Source (Recommended)

```bash
git clone https://github.com/anthropics/specter
cd specter
cargo install --path .
specter --version
```

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/anthropics/specter/main/install.sh | sh
```

## Prerequisites

Before using Specter, ensure you have:

1. **AI CLI Tools** (choose based on your needs):
   - [Gemini CLI](https://github.com/google/generative-ai) for proposal generation
   - [Codex CLI](https://github.com/openai/codex-cli) for code review and testing
   - [Claude Code](https://claude.ai/code) for interactive implementation

2. **API Keys** (set as environment variables):
   ```bash
   export GEMINI_API_KEY="your-gemini-key"
   export OPENAI_API_KEY="your-openai-key"
   export ANTHROPIC_API_KEY="your-anthropic-key"
   ```

3. **Development Tools** (for Rust projects):
   - Rust toolchain: `rustc`, `cargo`
   - Code quality tools: `cargo clippy`, `cargo test`
   - Security: `cargo-audit`, `semgrep` (optional, for review scripts)

> **Note**: Specter is language-agnostic but review scripts are currently optimized for Rust. Customize `specter/scripts/codex-review.sh` for other tech stacks.

## Quick Start

### 1. Initialize Project
```bash
cd your-project
specter init
```

### 2. Basic Workflow

```bash
# 1. Create a proposal (automatically runs challenge → reproposal loop)
specter proposal add-oauth "Add OAuth authentication"
# This single command:
#   - Generates proposal with Gemini
#   - Challenges it with Codex
#   - Refines it with Gemini (if issues found)
#   - Outputs a ready-to-implement plan

# 2. Implement the feature (interactive with Claude Code)
specter implement add-oauth
# Requires Claude Code Skills - opens interactive session

# 3. Review and test implementation
specter review add-oauth
# Generates tests and runs code review with Codex

# 4. Fix any issues found
specter fix add-oauth  # (if review found issues)

# 5. Archive and merge to main specs
specter archive add-oauth
```

### 3. Manual Steps (Advanced)

If you need more control, you can run individual phases:

```bash
# Run challenge separately (if not using automatic loop)
specter challenge add-oauth

# Manually trigger reproposal
specter reproposal add-oauth

# Refine with additional requirements
specter refine add-oauth "Add support for GitHub OAuth"
```

## Detailed Workflow

Specter follows a strict lifecycle to ensure quality.

### 1. Proposal Generation (Automated Loop)
**Primary Agent:** Gemini (configurable)

When you run `specter proposal`, the system automatically:
1. Generates initial proposal with full codebase context (up to 2M tokens)
2. Creates `proposal.md`, `tasks.md`, `diagrams.md`, and spec deltas
3. **Challenges** the proposal with Codex to find issues
4. **Refines** the proposal if HIGH severity issues are found (one automatic iteration)
5. **Outcome:** A vetted, high-quality spec ready for implementation

### 2. Implementation (Interactive)
**Agent:** Claude Code (Claude 3.5 Sonnet)

**Important**: Implementation requires Claude Code interactive environment.
- Executes the tasks defined in `tasks.md`
- Includes automatic review loop during implementation
- Updates `IMPLEMENTATION.md` with progress notes
- **Outcome:** Code changes applied to your project

### 3. Review & Testing
**Agent:** Codex (configurable)

- Generates targeted unit and integration tests based on specs
- Runs the project's test suite
- Performs security scanning and code quality checks
- **Outcome:** A `REVIEW.md` report with test results and findings

### 4. Archive (The Quality Gate)
**Agents:** Mixed (validation, Gemini, Codex)

The archive command is a rigorous **7-step process**:
1. **Validation**: Validates spec format and semantics using AST parsing (zero token cost)
2. **Delta Analysis**: Computes metrics and decides merge strategy (zero token cost)
3. **Backup**: Creates safety snapshot of `specter/specs/` before modification
4. **Spec Merging**: Gemini merges spec deltas into main specs directory
5. **Changelog Generation**: Gemini updates `specter/specs/CHANGELOG.md` automatically
6. **Quality Review**: Codex reviews merged specs and changelog for hallucinations or omissions
7. **Archive/Rollback**: If approved, moves to `archive/`; if review fails, rolls back to backup

## Project Structure

```
project/
├── specter/
│   ├── config.toml              # Configuration
│   ├── specs/                   # Source of Truth: Main specifications
│   │   ├── CHANGELOG.md         # Auto-generated changelog
│   │   └── auth/spec.md
│   ├── changes/                 # Active Work
│   │   └── add-oauth/
│   │       ├── proposal.md      # The Plan (Why, What, Impact)
│   │       ├── tasks.md         # Implementation checklist
│   │       ├── diagrams.md      # Architecture diagrams
│   │       ├── specs/           # Spec Deltas (changes to merge)
│   │       ├── GEMINI.md        # Auto-generated context for Gemini
│   │       ├── AGENTS.md        # Auto-generated context for Codex
│   │       ├── CHALLENGE.md     # Challenge review feedback
│   │       ├── REVIEW.md        # Code review results
│   │       ├── IMPLEMENTATION.md # Implementation notes
│   │       └── ARCHIVE_REVIEW.md # Final quality gate report
│   ├── archive/                 # Completed History
│   │   └── 20260113-add-oauth/
│   └── scripts/                 # AI Bridge Scripts (customizable)
├── .claude/
│   └── skills/                  # Claude Code Skills (installed by init)
└── .gemini/
    └── commands/specter/        # Gemini command definitions
```

## Commands

### Core Commands

| Command | Description |
|---------|-------------|
| `specter init` | Initialize Specter in current directory |
| `specter proposal <id> "<desc>"` | Generate proposal (auto-runs challenge + reproposal) |
| `specter challenge <id>` | Manually challenge a proposal with Codex |
| `specter reproposal <id>` | Manually refine proposal based on challenge |
| `specter refine <id> "<requirements>"` | Add requirements to existing proposal |
| `specter implement <id>` | Implement changes (requires Claude Code) |
| `specter review <id>` | Review implementation and generate tests |
| `specter fix <id>` | Fix issues found during review |
| `specter archive <id>` | Archive completed change (7-step quality gate) |

### Utility Commands

| Command | Description |
|---------|-------------|
| `specter list` | List active changes |
| `specter list --archived` | List archived changes |
| `specter status <id>` | Show detailed change status |

## Architecture

Specter operates through three layers:

1. **CLI Layer**: Rust-based command parsing and validation (clap)
2. **Orchestration Layer**: Manages state, session resumption, and tool chaining (tokio)
3. **AI Integration Layer**:
   - Project-specific context generation (GEMINI.md, AGENTS.md)
   - Specialized prompts for different models
   - Safety sandboxing for script execution

**Key optimizations**:
- Session resumption via cached contexts
- Skeleton-based generation to reduce token usage
- Change-id conflict resolution before LLM calls
- Zero-cost validation with AST-based parsing using pulldown-cmark

## Technical Stack

Built with Rust for:
- **Performance**: Compiled binary with minimal overhead
- **Type safety**: Compile-time guarantees for correctness
- **Portability**: Single binary, no runtime dependencies
- **Reliability**: Robust error handling with anyhow/thiserror

## Documentation

- [Installation Guide](INSTALL.md) - Detailed installation instructions
- [Architecture Documentation](CLAUDE.md) - Technical architecture and design
- [Configuration](specter/config.toml.example) - Configuration options

## Contributing

Contributions are welcome! Please follow standard Rust conventions. All new features should:
1. Pass `cargo test`
2. Pass `cargo clippy` with no warnings
3. Follow the internal Specter workflow (Proposal → Implement → Review)

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Specter enables cost-effective, AI-assisted spec-driven development through intelligent orchestration and iterative refinement.**
