# Specter

**Spec**-driven Development Orches**ter** (Orchestrator)

A Rust-based tool for spec-driven development with AI-assisted iterative proposal refinement.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Specter orchestrates multiple AI tools to optimize cost and quality in software development:

- **Gemini** (2M context, cost-effective) - Codebase exploration and proposal generation
- **Codex** (code-specialized) - Proposal review and test generation
- **Claude** - Precise implementation

The tool integrates with Claude Code via Skills, enabling complete workflow management within interactive sessions.

## Key Features

- **Automated Challenge Phase**: AI reviews proposals against existing codebase before implementation
- **Iterative Refinement**: Proposal → Challenge → Reproposal loop with session caching
- **Conflict Resolution**: Automatic change-id conflict detection and resolution before LLM calls
- **Cost Optimization**: 70-75% cost reduction compared to single-AI approaches
- **Verification**: Automated test generation and validation

## Installation

### From Source (Recommended)

```bash
git clone https://github.com/your-repo/specter
cd specter
cargo install --path .
specter --version
```

### Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/your-repo/specter/main/install.sh | sh
```

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

## Quick Start

### Initialize Project

```bash
cd your-project
specter init
```

This creates:
```
.claude/skills/         # 6 Claude Code Skills
specter/
  ├── config.toml       # Configuration
  ├── specs/            # Specifications
  ├── changes/          # Active changes
  └── scripts/          # AI integration
```

### Basic Usage

Run commands directly in Claude Code:

```
/specter:proposal add-oauth "Add OAuth authentication"
/specter:challenge add-oauth
/specter:reproposal add-oauth
/specter:implement add-oauth
/specter:verify add-oauth
/specter:archive add-oauth
```

Or via CLI:

```bash
specter proposal add-oauth "Add OAuth authentication"
specter challenge add-oauth
specter reproposal add-oauth
```

## Workflow

### 1. Proposal Generation
```bash
specter proposal <change-id> "<description>"
```
- Gemini explores codebase with 2M context window
- Generates proposal.md, tasks.md, diagrams.md, and spec deltas
- Intelligent conflict resolution for duplicate change-ids

### 2. Challenge Phase
```bash
specter challenge <change-id>
```
- Codex analyzes proposal against existing code
- Identifies internal inconsistencies (HIGH priority)
- Notes code alignment issues (MEDIUM/LOW priority)
- Generates CHALLENGE.md with severity-tagged issues

### 3. Reproposal (Optional)
```bash
specter reproposal <change-id>
```
- Automatically fixes issues identified in challenge
- One automatic iteration with session resumption
- 41% token savings via cached context

### 4. Implementation
```bash
specter implement <change-id>
```
- Claude implements tasks from refined proposal
- Records progress in IMPLEMENTATION.md

### 5. Verification
```bash
specter verify <change-id>
```
- Codex generates tests from specifications
- Runs tests and reports results in VERIFICATION.md

### 6. Archive
```bash
specter archive <change-id>
```
- Archives completed change with timestamp

## Project Structure

```
project/
├── specter/
│   ├── config.toml              # Configuration
│   ├── specs/                   # Main specifications
│   │   └── auth/spec.md
│   ├── changes/                 # Active changes
│   │   └── add-oauth/
│   │       ├── proposal.md      # Why, what, impact
│   │       ├── tasks.md         # Implementation checklist
│   │       ├── diagrams.md      # Architecture diagrams
│   │       ├── specs/           # Spec deltas
│   │       ├── CHALLENGE.md     # Review feedback
│   │       ├── IMPLEMENTATION.md # Implementation notes
│   │       └── VERIFICATION.md  # Test results
│   ├── archive/                 # Completed changes
│   └── scripts/                 # AI integration scripts
├── .claude/
│   └── skills/                  # Claude Code Skills
└── .gemini/
    └── commands/specter/        # Gemini command definitions
```

## Commands

### Core Commands

| Command | Description |
|---------|-------------|
| `specter init` | Initialize Specter in current directory |
| `specter proposal <id> "<desc>"` | Generate proposal with conflict resolution |
| `specter challenge <id>` | Review proposal with Codex |
| `specter reproposal <id>` | Auto-fix issues from challenge |
| `specter implement <id>` | Implement changes |
| `specter verify <id>` | Generate and run tests |
| `specter archive <id>` | Archive completed change |

### Utility Commands

| Command | Description |
|---------|-------------|
| `specter list` | List active changes |
| `specter list --archived` | List archived changes |
| `specter status <id>` | Show change status |
| `specter fix <id>` | Fix verification failures |

## Cost Analysis

Approximate costs for a typical feature (100+ file codebase):

| Phase | Pure Claude | Specter | Savings |
|-------|-------------|---------|---------|
| Proposal generation | High | Low | 80% |
| Code review | High | Low | 75% |
| Implementation | Medium | Medium | 0% |
| Test generation | Medium | Low | 60% |
| **Total** | **$15-20** | **$4-5** | **70-75%** |

## Technical Stack

Built with Rust for:
- Performance: 10-20x faster than Node.js alternatives
- Type safety: Compile-time guarantees
- Portability: Single binary, no runtime dependencies
- Reliability: Robust error handling with anyhow/thiserror

## Architecture

Specter operates through three layers:

1. **CLI Layer**: Command parsing and validation (clap)
2. **Orchestration Layer**: Script execution and session management (tokio)
3. **AI Integration Layer**:
   - Gemini CLI with project-specific commands (.gemini/commands/)
   - Codex CLI with user-space prompts (~/.codex/prompts/)
   - Dynamic context generation per change (GEMINI.md, AGENTS.md)

Key optimizations:
- Session resumption for 41% token savings
- Skeleton-based generation for 10-15% token savings
- Change-id conflict resolution before LLM calls (zero token waste)

## Requirements

- Rust 1.70+ (for installation from source)
- Gemini CLI (for proposal generation)
- Codex CLI (for challenge and verification)
- Claude Code (for interactive Skills workflow)

## Documentation

- [Installation Guide](INSTALL.md)
- [Architecture Documentation](CLAUDE.md)
- [Design Document](/tmp/specter-design.md)

## Contributing

Contributions are welcome. Please follow standard Rust conventions and include tests for new features.

## License

MIT License - see LICENSE file for details.

---

**Specter enables cost-effective, AI-assisted spec-driven development through intelligent orchestration and iterative refinement.**
