# MCP Configuration Files

Stage-specific MCP server configurations for Claude Code CLI.

## Files

### `implement.json`

Configuration for the **Implement** stage (Claude).

**Tools exposed** (4 tools):
- `read_all_requirements` - Read all requirements from specs
- `read_implementation_summary` - Read implementation summary
- `list_changed_files` - List modified files
- `read_file` - Read spec content

**Usage**:
```bash
claude --mcp-config ./agentd/mcp-configs/implement.json
```

## Why Stage-Specific Configs?

Agentd's MCP server exposes 22 tools in total (14 core + 8 Mermaid). Different workflow stages only need a subset:

| Stage | Provider | Tools Needed | Config Method |
|-------|----------|--------------|---------------|
| **Plan** | Gemini | 22 (all) | `~/.config/gemini/config.toml` |
| **Challenge** | Codex | 5 | Codex profile: `agentd-challenge` |
| **Implement** | Claude | 4 | `implement.json` (this file) |
| **Review** | Codex | 3 | Codex profile: `agentd-review` |
| **Archive** | Gemini | 6 | Default config |

**Benefits**:
- ✅ Reduced cognitive load (4 tools vs 22)
- ✅ Faster tool selection
- ✅ Token savings
- ✅ Stage isolation

## Codex Profiles

For Codex stages (Challenge, Review), use profiles in `~/.codex/config.toml`:

```toml
[agentd-review]
model = "gpt-5.2-codex"
reasoning = "medium"

[agentd-review.mcp]
agentd = { command = "agentd", args = ["mcp-server", "--tools", "review"] }

[agentd-challenge]
model = "gpt-5.2-codex"
reasoning = "high"

[agentd-challenge.mcp]
agentd = { command = "agentd", args = ["mcp-server", "--tools", "challenge"] }
```

**Usage**:
```bash
codex --profile agentd-review
codex --profile agentd-challenge
```

## Testing

Test tool filtering:
```bash
# Should expose only 4 tools
agentd mcp-server --tools implement

# Should expose only 3 tools
agentd mcp-server --tools review

# Should expose all 22 tools
agentd mcp-server
```

## References

- [Dynamic Config Strategy](../knowledge/40-mcp/dynamic-config.md)
- [Claude Code MCP](../knowledge/40-mcp/claude-mcp.md)
- [Codex MCP](../knowledge/40-mcp/codex-mcp.md)
