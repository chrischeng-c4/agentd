# Gemini CLI Custom Commands

Reference: https://geminicli.com/docs/cli/custom-commands/

## File Structure & Locations

Custom commands are discovered from two locations in order of precedence:

1. **User commands (global):** `~/.gemini/commands/`
2. **Project commands (local):** `<project-root>/.gemini/commands/`

Project-level commands override identically-named global commands.

## Naming & Namespacing

Command names derive from file paths relative to the `commands` directory. Subdirectories create namespaced commands using colons:

| File Path | Command Name |
|-----------|--------------|
| `~/.gemini/commands/test.toml` | `/test` |
| `<project>/.gemini/commands/git/commit.toml` | `/git:commit` |
| `<project>/.gemini/commands/plan/gen-tasks.toml` | `/plan:gen-tasks` |

## TOML Format

Commands use `.toml` file format:

```toml
# Required
prompt = "The prompt that will be sent to the Gemini model"

# Optional
description = "Brief one-line description for /help menu"
```

## Argument Handling

### 1. `{{args}}` Placeholder

Replaces placeholder with user input:
- **Raw injection:** Arguments injected exactly as typed in main prompt body
- **Shell commands:** Arguments automatically shell-escaped inside `!{...}` blocks

### 2. Default Behavior

When `{{args}}` is absent, the CLI appends the full command to the prompt with two newlines.

### 3. Shell Command Execution `!{...}`

Executes shell commands and injects output:
```toml
prompt = """
Current git status:
!{git status}

Please analyze this repository.
"""
```

Features:
- Argument substitution with automatic shell-escaping
- Security confirmation before execution
- Error reporting includes stderr and exit codes
- Requires balanced braces within blocks

### 4. File Content Injection `@{...}`

Embeds file/directory content into prompts:
```toml
prompt = """
Review this code:
@{src/main.rs}
"""
```

Features:
- File injection: `@{path/to/file.txt}`
- Multimodal support for images, PDFs, audio, video
- Directory traversal respecting `.gitignore` and `.geminiignore`
- Processing occurs before shell commands and argument substitution

## Example: MCP-Enabled Command

```toml
# .gemini/commands/plan/gen-tasks.toml
description = "Generate tasks.md using MCP create_tasks tool"

prompt = """
## Task: Create tasks.md

Use the `create_tasks` MCP tool to generate tasks.md.

Read:
- proposal.md for scope
- specs/*.md for requirements

IMPORTANT: Use create_tasks MCP tool, NOT write_file.
"""
```

## Limitations

- Shell command content must have balanced braces
- Complex unbalanced brace commands require external script wrapping
- File injection paths must have balanced braces
- Multimodal support limited to specific formats
