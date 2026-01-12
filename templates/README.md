# Specter Skills Templates

These templates are embedded into the `specter` binary and installed when you run `specter init`.

## Installation

```bash
# In your project directory
specter init

# This creates:
.claude/skills/
  ├── specter-proposal/SKILL.md
  ├── specter-challenge/SKILL.md
  ├── specter-reproposal/SKILL.md
  ├── specter-implement/SKILL.md
  ├── specter-verify/SKILL.md
  └── specter-archive/SKILL.md
```

## Usage in Claude Code

After running `specter init`, you can use these skills directly in Claude Code:

```
/specter:proposal add-oauth "Add OAuth authentication"
/specter:challenge add-oauth
/specter:reproposal add-oauth
/specter:implement add-oauth
/specter:verify add-oauth
/specter:archive add-oauth
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

1. Find it in `.claude/skills/specter-*/SKILL.md`
2. Edit the SKILL.md file
3. Reload Claude Code
4. Use the skill with `/specter:*`

## Development

To add new skills:

1. Create `templates/skills/specter-newskill/SKILL.md`
2. Update `src/cli/init.rs` to include the new skill
3. Rebuild specter: `cargo build --release`
4. Run `specter init` in a test project

## Skill List

- **specter-proposal** - Generate proposal with Gemini (2M context)
- **specter-challenge** - Challenge proposal with Codex analysis
- **specter-reproposal** - Refine proposal based on feedback
- **specter-implement** - Implement tasks with Claude
- **specter-verify** - Generate tests and verify with Codex
- **specter-archive** - Archive completed change
