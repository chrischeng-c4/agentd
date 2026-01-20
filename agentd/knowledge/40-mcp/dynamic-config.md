# Dynamic MCP Configuration Strategy

Runtime MCP configuration allows different workflow stages to access only the tools they need.

## Problem Statement

Agentd's MCP server exposes **22 tools**:
- 14 core tools (proposal, spec, tasks, validation, knowledge, implementation)
- 8 Mermaid diagram tools (flowchart, sequence, state, class, ERD, mindmap, requirement, journey)

**Issues**:
- Implementation stage only needs 4 tools but sees all 22
- Review stage only needs 3 tools but sees all 22
- Increases LLM cognitive load and prompt token usage

## Solution Architecture

### Tool Filtering by Stage

Implement `--tools <stage>` flag in `agentd mcp-server`:

```bash
agentd mcp-server --tools plan        # 22 tools (all)
agentd mcp-server --tools implement   # 4 tools (impl only)
agentd mcp-server --tools review      # 3 tools (review only)
agentd mcp-server --tools challenge   # 5 tools (challenge only)
agentd mcp-server --tools archive     # 6 tools (knowledge + read)
```

### Tool Sets by Stage

| Stage | Provider | Tool Count | Tools |
|-------|----------|------------|-------|
| **Plan** | Gemini | 22 | All core + Mermaid |
| **Challenge** | Codex | 5 | read_file, list_specs, read_knowledge, validate_change |
| **Implement** | Claude | 4 | read_all_requirements, read_implementation_summary, list_changed_files, read_file |
| **Review** | Codex | 3 | validate_change, append_review, read_file |
| **Archive** | Gemini | 6 | read_knowledge, write_knowledge, read_file, list_specs |

## Implementation Strategy

### Step 1: Implement Tool Filtering in MCP Server

```rust
// src/mcp/tools/mod.rs
impl ToolRegistry {
    pub fn new_for_stage(stage: &str) -> Self {
        let tools = match stage {
            "plan" => Self::plan_tools(),
            "challenge" => Self::challenge_tools(),
            "implement" => Self::implement_tools(),
            "review" => Self::review_tools(),
            "archive" => Self::archive_tools(),
            _ => Self::all_tools(),
        };
        Self { tools }
    }

    fn plan_tools() -> Vec<ToolDefinition> {
        let mut tools = Self::core_tools();
        tools.extend(mermaid::definitions());
        tools
    }

    fn implement_tools() -> Vec<ToolDefinition> {
        vec![
            implementation::read_all_requirements_definition(),
            implementation::read_implementation_summary_definition(),
            implementation::list_changed_files_definition(),
            read::definition(),
        ]
    }

    fn review_tools() -> Vec<ToolDefinition> {
        vec![
            validate::definition(),
            proposal::append_review_definition(),
            read::definition(),
        ]
    }

    fn challenge_tools() -> Vec<ToolDefinition> {
        vec![
            read::definition(),
            read::list_specs_definition(),
            knowledge::read_definition(),
            validate::definition(),
        ]
    }

    fn archive_tools() -> Vec<ToolDefinition> {
        vec![
            knowledge::read_definition(),
            knowledge::write_definition(),
            read::definition(),
            read::list_specs_definition(),
        ]
    }
}
```

### Step 2: Update CLI to Accept --tools Flag

```rust
// src/cli/mcp_server.rs
#[derive(Parser)]
pub struct McpServerArgs {
    /// Filter tools by workflow stage
    #[arg(long, value_name = "STAGE")]
    tools: Option<String>,
}

pub async fn run(args: McpServerArgs) -> Result<()> {
    let registry = if let Some(stage) = args.tools {
        ToolRegistry::new_for_stage(&stage)
    } else {
        ToolRegistry::new()
    };

    // ... rest of MCP server implementation
}
```

### Step 3: Provider-Specific Integration

#### Claude (Implement Stage)

Create `agentd/mcp-configs/implement.json`:
```json
{
  "mcpServers": {
    "agentd": {
      "command": "agentd",
      "args": ["mcp-server", "--tools", "implement"]
    }
  }
}
```

Update orchestrator:
```rust
// src/orchestrator/claude.rs
let args = vec![
    "--mcp-config".to_string(),
    project_root.join("agentd/mcp-configs/implement.json").display().to_string(),
];
```

#### Codex (Review/Challenge Stages)

⚠️ **UPDATE**: Codex currently does **NOT support per-profile MCP servers** ([Issue #2800](https://github.com/openai/codex/issues/2800)).

**Current approach**: Use global MCP configuration with all tools:

`~/.codex/config.toml`:
```toml
model = "gpt-5.2-codex"
reasoning = "medium"

[mcp_servers.agentd]
command = "agentd"
args = ["mcp-server"]  # All 22 tools exposed
```

**Rationale**:
- Codex's **400K context window** can easily handle 22 tools
- Prompt engineering ensures correct tool usage
- Simpler than manual config switching
- Future-proof when per-profile MCP support lands

Update orchestrator (no special args):
```rust
// src/orchestrator/codex.rs
let args = vec![];  // Use default config
let prompt = "Review the implementation. Use validate_change and append_review tools.";
```

#### Gemini (Plan/Archive Stages)

Use default config (all tools):
```toml
# ~/.config/gemini/config.toml
[mcp.servers.agentd]
command = "agentd"
args = ["mcp-server"]  # No --tools flag, expose all
```

**Rationale**: Gemini has 2M context window, tool count impact is minimal.

## Benefits

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Tools in Implement | 22 | 4 | 82% reduction |
| Tools in Review | 22 | 3 | 86% reduction |
| Tools in Challenge | 22 | 5 | 77% reduction |
| LLM cognitive load | High | Low | ✅ Focused |
| Prompt token usage | High | Low | ✅ Efficient |

## Migration Path

1. ✅ Implement `--tools` flag in MCP server
2. ✅ Create stage-specific tool filtering functions
3. ✅ Create MCP config files for Claude stages
4. ✅ Add Codex profiles to config
5. ✅ Update orchestrators to use dynamic configs
6. ✅ Test each workflow stage independently
7. ✅ Document setup in README

## Testing Strategy

```bash
# Test tool filtering
agentd mcp-server --tools implement
# Should expose only: read_all_requirements, read_implementation_summary,
#                     list_changed_files, read_file

agentd mcp-server --tools review
# Should expose only: validate_change, append_review, read_file

# Test orchestrator integration
agentd implement test-change
# Should use implement.json config with 4 tools

agentd review test-change
# Should use agentd-review profile with 3 tools
```

## Future Enhancements

1. **Auto-generate MCP configs**: `agentd init` creates all stage-specific configs
2. **Codex profile injection**: Auto-add profiles to `~/.codex/config.toml`
3. **Validation**: Check that correct tools are available before running stage
4. **Telemetry**: Log which tools are actually used per stage
