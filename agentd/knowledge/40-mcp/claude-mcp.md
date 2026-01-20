# Claude Code MCP Configuration

Claude Code (CLI) supports runtime MCP configuration via command-line flags.

## References

- Official Docs: https://code.claude.com/docs/en/cli-reference

## Runtime MCP Configuration

### `--mcp-config` Flag

Load MCP servers from JSON files or strings (space-separated).

```bash
claude --mcp-config ./mcp.json
claude --mcp-config ./mcp1.json ./mcp2.json
```

**MCP Config Format** (JSON):
```json
{
  "mcpServers": {
    "agentd": {
      "command": "agentd",
      "args": ["mcp-server", "--tools", "implement"],
      "env": {}
    }
  }
}
```

### `--strict-mcp-config` Flag

Only use MCP servers from `--mcp-config`, ignoring all other MCP configurations (e.g., `~/.config/claude/mcp.json`).

```bash
claude --strict-mcp-config --mcp-config ./mcp-implement.json
```

## Agentd Integration Strategy

### Step 1: Create Stage-Specific MCP Configs

```bash
# agentd/mcp-configs/implement.json
{
  "mcpServers": {
    "agentd-impl": {
      "command": "agentd",
      "args": ["mcp-server", "--tools", "implement"]
    }
  }
}
```

### Step 2: Update Orchestrator to Use Dynamic Config

```rust
// src/orchestrator/claude.rs
pub async fn run_implement_spec(...) -> Result<(String, UsageMetrics)> {
    let mcp_config = project_root.join("agentd/mcp-configs/implement.json");

    let args = vec![
        "--mcp-config".to_string(),
        mcp_config.display().to_string(),
    ];

    self.runner.run_llm(LlmProvider::Claude, args, env, &prompt, resume).await
}
```

## Tool Sets by Stage

### Implement Stage (4 tools)
```json
{
  "mcpServers": {
    "agentd-impl": {
      "command": "agentd",
      "args": ["mcp-server", "--tools", "implement"]
    }
  }
}
```

**Tools exposed**:
- `read_all_requirements` - Read all requirements from specs
- `read_implementation_summary` - Read implementation summary
- `list_changed_files` - List modified files
- `read_file` - Read spec content

### Review Stage (Use base config)

Review is done by Codex, not Claude. Claude only resolves review issues, which needs similar tools to implement stage.

## Benefits

1. **Reduced Tool Count**: 4 tools instead of 22
2. **Faster Tool Selection**: Less cognitive load for Claude
3. **Token Savings**: Smaller prompt overhead
4. **Isolation**: Implementation can't accidentally call plan tools
