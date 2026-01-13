# Agentd Skills Templates

These templates are embedded into the `agentd` binary and installed when you run `agentd init`.

## Installation

```bash
# In your project directory
agentd init

# This creates:
.claude/skills/
  ├── agentd-proposal/SKILL.md
  ├── agentd-challenge/SKILL.md
  ├── agentd-reproposal/SKILL.md
  ├── agentd-implement/SKILL.md
  ├── agentd-verify/SKILL.md
  └── agentd-archive/SKILL.md
```

## Usage in Claude Code

After running `agentd init`, you can use these skills directly in Claude Code:

```
/agentd:proposal add-oauth "Add OAuth authentication"
/agentd:challenge add-oauth
/agentd:reproposal add-oauth
/agentd:implement add-oauth
/agentd:verify add-oauth
/agentd:archive add-oauth
```

## Skill Architecture

Each skill follows this pattern:

1. **Validate inputs** - Check required parameters
2. **Check state** - Ensure prerequisites are met
3. **Execute task** - Call AI tools or perform operations
4. **Verify results** - Ensure output is correct
5. **Display feedback** - Show results to user

## Customization

To customize a skill:

1. Find it in `.claude/skills/agentd-*/SKILL.md`
2. Edit the SKILL.md file
3. Reload Claude Code
4. Use the skill with `/agentd:*`

## Development

To add new skills:

1. Create `templates/skills/agentd-newskill/SKILL.md`
2. Update `src/cli/init.rs` to include the new skill
3. Rebuild agentd: `cargo build --release`
4. Run `agentd init` in a test project

## Skill List

- **agentd-proposal** - Generate proposal with Gemini (2M context)
- **agentd-challenge** - Challenge proposal with Codex analysis
- **agentd-reproposal** - Refine proposal based on feedback
- **agentd-implement** - Implement tasks with Claude
- **agentd-verify** - Generate tests and verify with Codex
- **agentd-archive** - Archive completed change
