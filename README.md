# Agentd

AI-powered spec-driven development orchestrator.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/agentd/main/install.sh | bash
```

To update to the latest version:

```bash
agentd update
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

# 2. Plan the change (Proposal → Challenge → Refine)
agentd plan add-oauth "Add OAuth authentication with Google"

# 3. Implement (opens Claude Code)
agentd implement add-oauth

# 4. Archive when done
agentd archive add-oauth
```

## Commands

### Workflow Commands

| Command | Description |
|---------|-------------|
| `agentd init` | Initialize Agentd in current project |
| `agentd plan <id> "<description>"` | Plan a change (Proposal → Challenge loop) |
| `agentd implement <id>` | Implement the change (requires Claude Code) |
| `agentd archive <id>` | Archive completed change |
| `agentd list` | List active changes |
| `agentd list --archived` | List archived changes |
| `agentd status <id>` | Show change status |

### Low-Level Commands (Advanced)

| Command | Description |
|---------|-------------|
| `agentd proposal <id>` | Create a new proposal |
| `agentd challenge <id>` | Challenge proposal with code review |
| `agentd reproposal <id>` | Refine proposal based on feedback |
| `agentd review <id>` | Review implementation and run tests |
| `agentd resolve-reviews <id>` | Fix issues found during review |
| `agentd validate-proposal <id>` | Validate proposal format |
| `agentd validate-challenge <id>` | Validate challenge format |
| `agentd view <id>` | Open plan viewer UI (requires `ui` feature) |

## Workflow

```
plan (proposal ⇄ challenge) → implement (code ⇄ review) → archive
```

1. **Plan**: Generate PRD/Specs (Gemini) and refine with reviews (Codex) until approved.
2. **Implement**: Write code (Claude Code) and resolve issues until verified.
3. **Archive**: Merge specs and archive the change.

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
| `/agentd:plan` | `agentd plan` | Plan a change (Proposal → Challenge) |
| `/agentd:impl` | `agentd implement` | Implement and verify change |
| `/agentd:archive` | `agentd archive` | Archive completed change |

### Deprecated Skills
Granular skills (e.g., `/agentd:proposal`, `/agentd:challenge`) are available but deprecated.

## Plan Viewer UI (Optional)

Agentd includes an optional native UI for viewing plans. Build with the `ui` feature to enable:

```bash
cargo build --features ui
```

Then open the viewer for any change:

```bash
agentd view <change-id>
```

The viewer provides:
- Rendered Markdown with Mermaid diagrams
- Syntax-highlighted YAML state files
- Annotation support for human review comments
- Navigation between proposal, challenge, and state files

The viewer auto-opens when a proposal is approved (if built with `ui` feature).

## License

MIT
