# Agentd

AI-powered spec-driven development orchestrator.

## Installation

```bash
brew tap chrischeng-c4/tap
brew install agentd
```

## Prerequisites

Set up your AI CLI tools and API keys:

```bash
export GEMINI_API_KEY="your-gemini-key"
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
```

## Quick Start

```bash
# 1. Initialize in your project
cd your-project
agentd init

# 2. Create a proposal
agentd proposal add-oauth "Add OAuth authentication with Google"

# 3. Implement (opens Claude Code)
agentd implement add-oauth

# 4. Review and test
agentd review add-oauth

# 5. Archive when done
agentd archive add-oauth
```

## Commands

### Workflow Commands

| Command | Description |
|---------|-------------|
| `agentd init` | Initialize Agentd in current project |
| `agentd proposal <id> "<description>"` | Create a new proposal |
| `agentd challenge-proposal <id>` | Challenge proposal with code review |
| `agentd reproposal <id>` | Refine proposal based on challenge feedback |
| `agentd implement <id>` | Implement the change (requires Claude Code) |
| `agentd review <id>` | Review implementation and run tests |
| `agentd fix <id>` | Fix issues found during review |
| `agentd archive <id>` | Archive completed change |

### Validation Commands

| Command | Description |
|---------|-------------|
| `agentd validate-proposal <id>` | Validate proposal format |
| `agentd validate-challenge <id>` | Validate challenge format |

Options:
- `--strict` - Treat warnings as errors
- `--verbose` - Show detailed error locations
- `--json` - Output as JSON

### Utility Commands

| Command | Description |
|---------|-------------|
| `agentd list` | List active changes |
| `agentd list --archived` | List archived changes |
| `agentd status <id>` | Show change status |

## Workflow

```
proposal → challenge → reproposal → implement → review → fix → archive
```

1. **Proposal**: Generate PRD, tasks, and specs using Gemini
2. **Challenge**: Review proposal for issues using Codex
3. **Reproposal**: Refine based on feedback (automatic if issues found)
4. **Implement**: Write code using Claude Code
5. **Review**: Run tests and code review
6. **Fix**: Address any issues
7. **Archive**: Merge specs and archive the change

## Project Structure

After `agentd init`:

```
your-project/
├── agentd/
│   ├── config.toml          # Configuration
│   ├── specs/               # Main specifications
│   ├── changes/             # Active changes
│   │   └── <change-id>/
│   │       ├── proposal.md  # PRD
│   │       ├── tasks.md     # Implementation tasks
│   │       ├── specs/       # Technical design
│   │       └── CHALLENGE.md # Review feedback
│   └── archive/             # Completed changes
├── .claude/skills/          # Claude Code skills
└── .gemini/commands/        # Gemini commands
```

## Configuration

Edit `agentd/config.toml` to customize:

```toml
project_name = "my-project"

[gemini]
command = "gemini"
default = "flash"

[codex]
command = "codex"
default = "balanced"

[claude]
command = "claude"
default = "balanced"
```

## Claude Code Skills

After initialization, use these skills in Claude Code:

| Skill | CLI Equivalent | Description |
|-------|----------------|-------------|
| `/agentd:proposal` | `agentd proposal` | Generate proposal |
| `/agentd:challenge` | `agentd challenge-proposal` | Challenge proposal |
| `/agentd:reproposal` | `agentd reproposal` | Refine proposal |
| `/agentd:implement` | `agentd implement` | Implement change |
| `/agentd:review` | `agentd review` | Review implementation |
| `/agentd:fix` | `agentd fix` | Fix issues |
| `/agentd:archive` | `agentd archive` | Archive change |

## License

MIT
