---
title: Claude Code Skills
source: https://code.claude.com/docs/en/skills
date: 2026-01-19
updated: 2026-01-19
---

# Agent Skills

Agent Skills are markdown files that teach Claude how to do something specific. They enable automatic, context-aware capability extensions in Claude Code. When you ask Claude something matching a Skill's purpose, Claude automatically applies it.

**Examples of Skills:**
- Reviewing PRs using team standards
- Generating commit messages in preferred format
- Querying company database schema
- Explaining code with visual diagrams

## How Skills Work

Skills are **model-invoked** - Claude decides which to use based on your request. The process:

1. **Discovery**: Claude loads only the name and description of each available Skill at startup
2. **Activation**: When your request matches a Skill's description, Claude asks to use it
3. **Execution**: Claude follows the Skill's instructions, loading referenced files as needed

## Creating Your First Skill

### Step 1: Create Directory
```bash
mkdir -p ~/.claude/skills/explaining-code
```

### Step 2: Create SKILL.md
Every Skill needs a `SKILL.md` file with YAML metadata and markdown instructions:

```markdown
---
name: explaining-code
description: Explains code with visual diagrams and analogies. Use when explaining how code works, teaching about a codebase, or when the user asks "how does this work?"
---

When explaining code, always include:

1. **Start with an analogy**: Compare the code to something from everyday life
2. **Draw a diagram**: Use ASCII art to show the flow, structure, or relationships
3. **Walk through the code**: Explain step-by-step what happens
4. **Highlight a gotcha**: What's a common mistake or misconception?

Keep explanations conversational. For complex concepts, use multiple analogies.
```

### Step 3: Verify & Test
```
What Skills are available?
```

## Where Skills Live

| Location | Path | Applies to |
|----------|------|-----------|
| **Enterprise** | See managed settings | All users in organization |
| **Personal** | `~/.claude/skills/` | You, across all projects |
| **Project** | `.claude/skills/` | Anyone in repository |
| **Plugin** | Bundled with plugins | Anyone with plugin installed |

**Priority**: Enterprise > Personal > Project > Plugin

## SKILL.md Configuration

### Required Metadata Fields

```yaml
---
name: your-skill-name              # lowercase, letters/numbers/hyphens, max 64 chars
description: What it does          # max 1024 chars - crucial for triggering
---
```

### Optional Metadata Fields

| Field | Purpose |
|-------|---------|
| `allowed-tools` | Restrict which tools Claude can use (comma-separated or YAML list) |
| `model` | Specify model to use when Skill is active |
| `context: fork` | Run in isolated sub-agent context with separate conversation history |
| `agent` | Agent type to use with `context: fork` |
| `hooks` | Define PreToolUse, PostToolUse, or Stop event handlers |
| `user-invocable` | Controls Skill visibility in slash menu (default: true) |

### String Substitutions

Available in Skill content:

```
$ARGUMENTS          # All arguments passed when invoking
${CLAUDE_SESSION_ID} # Current session ID
```

## Skill Structure

### Simple Single-File Skill

```
commit-helper/
└── SKILL.md
```

### Multi-File Skill with Progressive Disclosure

```
my-skill/
├── SKILL.md                    # Overview and navigation
├── reference.md                # Detailed docs (loaded when needed)
├── examples.md                 # Usage examples
└── scripts/
    └── helper.py               # Utility scripts (executed, not loaded)
```

**Benefits of progressive disclosure:**
- Keep `SKILL.md` under 500 lines for optimal performance
- Claude loads additional files only when needed
- Utility scripts execute without consuming context tokens

Link supporting files from `SKILL.md`:

```markdown
## Additional resources

- For complete API details, see [reference.md](reference.md)
- For usage examples, see [examples.md](examples.md)

## Utility scripts

Run the validation script:
```bash
python scripts/validate_form.py input.pdf
```
```

## Restricting Tool Access

Use `allowed-tools` to limit capabilities:

```yaml
---
name: reading-files-safely
description: Read files without making changes
allowed-tools: Read, Grep, Glob
---
```

Or YAML style:

```yaml
allowed-tools:
  - Read
  - Grep
  - Glob
```

## Forked Context Skills

Run complex operations in isolated sub-agent context:

```yaml
---
name: code-analysis
description: Analyze code quality and generate reports
context: fork
---
```

## Skill Visibility Control

### `user-invocable` Field

| Setting | Slash Menu | `Skill` Tool | Auto-discovery | Use Case |
|---------|-----------|-------------|----------------|----------|
| `true` (default) | Visible | Allowed | Yes | Users can invoke directly |
| `false` | Hidden | Allowed | Yes | Claude uses but users don't see |
| `disable-model-invocation: true` | Visible | Blocked | Yes | Users invoke, not Claude |

**Example - Model-only Skill:**

```yaml
---
name: internal-review-standards
description: Apply internal code review standards when reviewing pull requests
user-invocable: false
---
```

## Skills and Subagents

### Give Subagent Access to Skills

In `.claude/agents/code-reviewer.md`:

```yaml
---
name: code-reviewer
description: Review code for quality and best practices
skills: pr-review, security-check
---
```

**Note**: Built-in agents (Explore, Plan, general-purpose) don't inherit Skills. Only custom subagents with explicit `skills` field can use them.

### Run Skill in Forked Subagent

```yaml
---
name: complex-task
description: Perform complex multi-step operation
context: fork
agent: Explore
---
```

## Distributing Skills

1. **Project Skills**: Commit `.claude/skills/` to version control
2. **Plugins**: Create `skills/` directory in plugin with `SKILL.md` files
3. **Managed**: Deploy organization-wide through managed settings

## Troubleshooting

### Skill Not Triggering
- Write descriptive, keyword-rich descriptions
- Include trigger terms users would naturally say

### Skill Doesn't Load
- Verify correct file path and `SKILL.md` filename (case-sensitive)
- Check YAML syntax (no tabs, proper indentation)
- Run `claude --debug` to see errors

### Script Errors
- Verify external packages are installed
- Ensure scripts have execute permissions: `chmod +x scripts/*.py`
- Use forward slashes in paths

### Multiple Skills Conflict
- Make descriptions distinct with specific trigger terms
- Avoid overlapping keywords between similar Skills

## Skills vs. Other Options

| Use This | When You Want To |
|----------|------------------|
| **Skills** | Give Claude specialized knowledge (auto-triggered) |
| **Slash commands** | Create reusable prompts you type explicitly |
| **CLAUDE.md** | Set project-wide instructions |
| **Subagents** | Delegate tasks with separate context/tools |
| **Hooks** | Run scripts on specific events |
| **MCP servers** | Connect Claude to external tools/data |

## Best Practices

- **Keep descriptions keyword-rich** for better triggering
- **Progressive disclosure**: Put essential info in `SKILL.md`, detailed reference in separate files
- **Bundle utility scripts**: Let Claude run scripts rather than read them
- **Validate YAML** syntax carefully
- **Test triggering** by asking questions matching the description
- **Limit context**: Keep `SKILL.md` under 500 lines