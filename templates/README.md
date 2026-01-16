# Agentd Skills Templates

These templates are embedded into the `agentd` binary and installed when you run `agentd init`.

## Installation

```bash
# In your project directory
agentd init

# This creates:
.claude/skills/
  ├── agentd-plan/SKILL.md      # Primary workflow entry
  ├── agentd-impl/SKILL.md      # Primary workflow entry
  ├── agentd-archive/SKILL.md   # Primary workflow entry
  ├── agentd-proposal/SKILL.md  # (deprecated)
  ├── agentd-challenge/SKILL.md # (deprecated)
  └── ...
```

## Usage in Claude Code

After running `agentd init`, use the high-level workflow skills:

```
# Planning phase (proposal + challenge + auto-reproposal)
/agentd:plan add-oauth "Add OAuth authentication"

# Implementation phase (implement + review + auto-fix)
/agentd:impl add-oauth

# Archive completed change
/agentd:archive add-oauth
```

### Deprecated Skills

The following granular skills are deprecated but still available:

- `/agentd:proposal` - Use `/agentd:plan` instead
- `/agentd:challenge` - Use `/agentd:plan` instead
- `/agentd:reproposal` - Use `/agentd:plan` instead
- `/agentd:implement` - Use `/agentd:impl` instead
- `/agentd:review` - Use `/agentd:impl` instead
- `/agentd:resolve-reviews` - Use `/agentd:impl` instead

## Skill Architecture

Each skill follows this pattern:

1. **Check state** - Read `STATE.yaml` phase
2. **Determine action** - Based on phase, decide next step
3. **Execute task** - Call appropriate `agentd` CLI command
4. **Display feedback** - Show results to user

## Customization

To customize a skill:

1. Find it in `.claude/skills/agentd-*/SKILL.md`
2. Edit the SKILL.md file
3. Reload Claude Code
