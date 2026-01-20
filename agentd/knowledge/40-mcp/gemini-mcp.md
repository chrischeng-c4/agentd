# Google Gemini MCP Configuration

Gemini CLI supports MCP servers but currently lacks runtime configuration flags.

## References

- Configuration: https://geminicli.com/docs/get-started/configuration/
- MCP Server: https://geminicli.com/docs/tools/mcp-server/

## Current Limitations

**No Runtime Override**: Gemini CLI does not currently support flags like `--mcp-config` or `--profile` to dynamically load MCP servers at runtime.

**Static Configuration**: MCP servers are configured in `~/.config/gemini/config.toml` and loaded on startup.

### Config File Structure

`~/.config/gemini/config.toml`:
```toml
[mcp.servers.agentd]
command = "agentd"
args = ["mcp-server"]
```

## Why This Is Acceptable

### Large Context Window

Gemini's **2M token context window** makes tool count less critical:

| Stage | Tools Needed | Impact |
|-------|--------------|--------|
| **Plan** | 22 (14 core + 8 Mermaid) | ✅ Primary use case |
| **Archive** | 6 (knowledge + read) | ✅ Can handle 22 tools easily |

### Plan Stage Tool Requirements

Plan stage **legitimately needs all tools**:
- ✅ Core tools: create_proposal, create_spec, create_tasks
- ✅ Mermaid tools: 8 diagram generators for spec visualization
- ✅ Knowledge tools: read_knowledge, list_knowledge
- ✅ Read tools: read_file, list_specs

### Archive Stage Tolerance

Archive stage merges specs to knowledge base:
- Needs: read_knowledge, write_knowledge, read_file, list_specs
- Can tolerate: All 22 tools exposed (won't use unnecessary ones)
- Impact: Minimal due to large context window

## Current Strategy

**No Dynamic Configuration Needed** for Gemini:

```toml
# ~/.config/gemini/config.toml
[mcp.servers.agentd]
command = "agentd"
args = ["mcp-server"]  # No --tools filter, expose all 22 tools
```

**Rationale**:
1. Plan stage uses Gemini and needs all 22 tools
2. Archive stage uses Gemini and can tolerate 22 tools
3. Implement/Review stages use Claude/Codex (different MCP configs)

## Future Considerations

If Gemini CLI adds `--mcp-config` or `--profile` support in the future, consider:

1. **Plan Profile**: All 22 tools
2. **Archive Profile**: Knowledge + read tools only (6 tools)

This would further optimize token usage, but is not critical given the 2M context window.

## Alternative: Subcommand-Based MCP Server

If optimization is needed now, use different MCP server binaries:

```toml
# Plan configuration
[mcp.servers.agentd-plan]
command = "agentd"
args = ["mcp-server", "--tools", "plan"]

# Archive configuration (would need manual config switching)
[mcp.servers.agentd-archive]
command = "agentd"
args = ["mcp-server", "--tools", "archive"]
```

**Problem**: Requires manual config file editing between plan and archive stages. Not recommended unless token usage becomes critical.

## Recommendation

**Keep current approach**: Expose all 22 tools to Gemini. The large context window makes this acceptable, and plan stage genuinely needs all tools.
