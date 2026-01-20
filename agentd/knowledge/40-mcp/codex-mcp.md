# OpenAI Codex MCP Configuration

⚠️ **IMPORTANT**: Codex currently does **NOT** support `--profile` flag with MCP servers. This is a known limitation.

## References

- MCP Overview: https://developers.openai.com/codex/mcp
- CLI Reference: https://developers.openai.com/codex/cli/reference
- Advanced Config: https://developers.openai.com/codex/config-advanced
- GitHub Issue #2800: https://github.com/openai/codex/issues/2800 (Profile support for MCP mode)

## Current Status (January 2026)

### What Works ✅
- `codex exec --profile=myprofile` - Works in EXEC mode
- `codex --profile=myprofile` (TUI mode) - Works in interactive mode
- Global MCP configuration in `~/.codex/config.toml`

### What Doesn't Work ❌
- ~~`codex mcp --profile=myprofile`~~ - **Does NOT work**
- ~~Per-profile MCP server configurations~~ - **Not supported yet**

**Status**: Feature request open ([Issue #2800](https://github.com/openai/codex/issues/2800)), PR #3106 in development but not merged.

## Current MCP Configuration Format

### Global MCP Servers (No Profile Support)

`~/.codex/config.toml`:
```toml
# Global configuration (applies to all codex invocations)
model = "gpt-5.2-codex"
reasoning = "medium"

# MCP servers - shared across all profiles
[mcp_servers.agentd]
command = "agentd"
args = ["mcp-server"]

# Additional MCP servers
[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/files"]
```

**Key Points**:
- Use `[mcp_servers.<name>]` not `[mcp.servers.<name>]`
- All MCP servers are global (no per-profile isolation)
- Cannot load different MCP tools per Codex invocation

## Workaround for Agentd

Since per-profile MCP is not supported, we have two options:

### Option A: Global Config with Manual Switching (Not Recommended)

Manually edit `~/.codex/config.toml` before each stage:

```bash
# Before review stage
vim ~/.codex/config.toml
# Change args to ["mcp-server", "--tools", "review"]

codex  # Run review

# Before challenge stage
vim ~/.codex/config.toml
# Change args to ["mcp-server", "--tools", "challenge"]

codex  # Run challenge
```

**Problems**: Error-prone, not automated, requires manual intervention.

### Option B: Skip Tool Filtering for Codex (Recommended)

Accept that Codex sees all 22 tools:

```toml
[mcp_servers.agentd]
command = "agentd"
args = ["mcp-server"]  # No --tools flag, expose all tools
```

**Rationale**:
- Codex has **400K context window** (sufficient for all tools)
- 22 tools is easily manageable for code review tasks
- Codex will only call relevant tools based on prompt
- Simpler configuration, no manual switching

## Tool Usage by Stage

Even with all 22 tools exposed, prompt engineering ensures correct tool usage:

### Review Stage
**Prompt**: "Review the code in change X. Use validate_change and append_review."
**Expected tools**: validate_change, append_review, read_file
**Won't use**: create_proposal, Mermaid tools (irrelevant to review)

### Challenge Stage
**Prompt**: "Challenge the proposal in change X by reading specs and knowledge base."
**Expected tools**: read_file, list_specs, read_knowledge
**Won't use**: create_proposal, Mermaid tools (irrelevant to challenge)

## Future: When Per-Profile MCP is Supported

If/when Issue #2800 is resolved, we can use:

```toml
[profiles.review]
model = "gpt-5.2-codex"
reasoning = "medium"

[profiles.review.mcp_servers.agentd]
command = "agentd"
args = ["mcp-server", "--tools", "review"]

[profiles.challenge]
model = "gpt-5.2-codex"
reasoning = "high"

[profiles.challenge.mcp_servers.agentd]
command = "agentd"
args = ["mcp-server", "--tools", "challenge"]
```

Then orchestrator could use:
```rust
codex --profile review     // 3 tools only
codex --profile challenge  // 5 tools only
```

## Agentd Integration Strategy (Current)

### Configuration

`~/.codex/config.toml`:
```toml
model = "gpt-5.2-codex"
reasoning = "medium"

[mcp_servers.agentd]
command = "agentd"
args = ["mcp-server"]  # All tools (22)
```

### Orchestrator Implementation

No special args needed - use default Codex invocation:

```rust
// src/orchestrator/codex.rs
pub async fn run_review(...) -> Result<(String, UsageMetrics)> {
    // No --profile flag, Codex uses default config with all MCP tools
    let args = vec![];

    // Prompt engineering ensures correct tool usage
    let prompt = "Review the implementation. Use validate_change and append_review tools.";

    self.runner.run_llm(LlmProvider::Codex, args, env, &prompt, resume).await
}
```

## Benefits of Current Approach

1. **No Manual Switching**: Configuration is static
2. **Prompt-Driven Tool Selection**: LLM naturally picks relevant tools
3. **Simpler Setup**: Single MCP configuration for all Codex stages
4. **Future-Proof**: When per-profile MCP lands, easy to migrate

## Monitoring

Track which tools Codex actually calls:
```bash
# Enable MCP logging
export CODEX_MCP_DEBUG=1

# Check which tools were called
grep "Calling tool:" ~/.codex/logs/mcp.log
```

If Codex frequently calls irrelevant tools, consider adding explicit prompts like:
- "Do NOT use Mermaid diagram tools"
- "Only use validation and review tools"
