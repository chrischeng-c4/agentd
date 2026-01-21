# MCP Tools vs CLI Commands: Complete Comparison

This document provides a comprehensive mapping between MCP (Model Context Protocol) tools and their equivalent CLI commands.

## Quick Summary

- **27 MCP tools total**
- **14 CLI commands** implemented (9 core + 5 specialized)
- **8 Mermaid tools** remain MCP-only (LLMs generate diagrams directly)
- **Shared service layer** ensures consistency

## Complete Tool/Command Mapping

### Knowledge Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `read_knowledge` | `agentd knowledge read <path>` | 1 | ✅ Implemented |
| `list_knowledge` | `agentd knowledge list [path]` | 1 | ✅ Implemented |
| `write_knowledge` | `agentd knowledge write <path> --json-file <file>` | 3 | ✅ Implemented |
| `write_main_spec` | `agentd knowledge write-spec <path> --content-file <file>` | 3 | ✅ Implemented |

**Examples**:
```bash
# Read knowledge file
agentd knowledge read 00-architecture/index.md

# List all knowledge files
agentd knowledge list

# Write knowledge document
agentd knowledge write 30-claude/mcp-overview.md --json-file knowledge.json

# Write spec to main directory (archive workflow)
agentd knowledge write-spec oauth-flow.md --content-file spec.md
```

---

### File Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `read_file` | `agentd file read <change-id> [file]` | 1 | ✅ Implemented |
| `list_specs` | `agentd spec list <change-id>` | 1 | ✅ Implemented |

**Examples**:
```bash
# Read proposal
agentd file read my-change proposal

# Read tasks
agentd file read my-change tasks

# Read specific spec
agentd file read my-change oauth-spec

# List all specs
agentd spec list my-change
```

---

### Proposal Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_proposal` | `agentd proposal create <id> --json-file <file>` | 2 | ✅ Implemented |
| `append_review` | `agentd proposal review <id> --json-file <file>` | 2 | ✅ Implemented |

**Examples**:
```bash
# Create proposal
agentd proposal create my-change --json-file proposal.json

# Add review
agentd proposal review my-change --json-file review.json
```

---

### Spec Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_spec` | `agentd spec create <id> <spec-id> --json-file <file>` | 2 | ✅ Implemented |

**Example**:
```bash
# Create spec
agentd spec create my-change oauth-spec --json-file spec.json
```

---

### Tasks Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_tasks` | `agentd tasks create <id> --json-file <file>` | 2 | ✅ Implemented |

**Example**:
```bash
# Create tasks
agentd tasks create my-change --json-file tasks.json
```

---

### Implementation Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `read_all_requirements` | `agentd implementation read-all <id>` | 3 | ✅ Implemented |
| `list_changed_files` | `agentd implementation list-files <id> [options]` | 3 | ✅ Implemented |
| `read_implementation_summary` | ❌ No CLI equivalent | - | MCP only |

**Notes**:
- `read_implementation_summary` provides git diff + commit log
- Can be replaced by direct git commands if needed
- Primary use case is for MCP-based workflows

**Examples**:
```bash
# Read all requirements (proposal + tasks + specs)
agentd implementation read-all my-change

# List changed files
agentd implementation list-files my-change

# Filter by path
agentd implementation list-files my-change --filter src/

# Compare against different branch
agentd implementation list-files my-change --base-branch develop
```

---

### Clarifications Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_clarifications` | `agentd clarifications create <id> --json-file <file>` | 3 | ✅ Implemented |

**Example**:
```bash
# Create clarifications
agentd clarifications create my-change --json-file clarifications.json
```

---

### Mermaid Diagram Generators

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `generate_mermaid_flowchart` | ❌ No CLI | - | MCP only |
| `generate_mermaid_sequence` | ❌ No CLI | - | MCP only |
| `generate_mermaid_class` | ❌ No CLI | - | MCP only |
| `generate_mermaid_state` | ❌ No CLI | - | MCP only |
| `generate_mermaid_erd` | ❌ No CLI | - | MCP only |
| `generate_mermaid_mindmap` | ❌ No CLI | - | MCP only |
| `generate_mermaid_journey` | ❌ No CLI | - | MCP only |
| `generate_mermaid_requirement` | ❌ No CLI | - | MCP only |

**Rationale for MCP-only**:
- LLMs can generate Mermaid code directly in markdown
- Primarily used through MCP during planning
- CLI would add ~8 commands with limited practical value
- Human users rarely need programmatic diagram generation

---

## Command Categories

### Read-Only Commands (Phase 1)

Fast, simple operations with no JSON files needed:

```bash
agentd knowledge list
agentd knowledge read <path>
agentd spec list <id>
agentd file read <id> [file]
```

### Creation Commands (Phase 2 & 3)

Complex operations using `--json-file` for structured input:

```bash
agentd proposal create <id> --json-file proposal.json
agentd proposal review <id> --json-file review.json
agentd spec create <id> <spec-id> --json-file spec.json
agentd tasks create <id> --json-file tasks.json
agentd clarifications create <id> --json-file clarifications.json
agentd knowledge write <path> --json-file knowledge.json
```

### Specialized Commands (Phase 3)

Advanced workflow support:

```bash
agentd implementation read-all <id>
agentd implementation list-files <id> [--filter <pattern>]
agentd knowledge write-spec <path> --content-file spec.md
```

---

## Interface Comparison

### Input Formats

| Aspect | MCP Tools | CLI Commands |
|--------|-----------|--------------|
| Simple inputs | JSON parameters via stdin | Command-line arguments |
| Complex inputs | JSON objects via stdin | JSON files on disk (`--json-file`) |
| File content | Embedded in JSON | Separate file (`--content-file`) |

### Output Formats

| Aspect | MCP Tools | CLI Commands |
|--------|-----------|--------------|
| Success | JSON-RPC response with result | Plain text with formatting |
| Errors | JSON-RPC error object | stderr + non-zero exit code |
| Formatting | Wrapped in XML tags (e.g., `<result>`) | Direct output with ANSI colors |

### Tool Discovery

| Aspect | MCP Tools | CLI Commands |
|--------|-----------|--------------|
| List tools | `tools/list` JSON-RPC method | `agentd --help` |
| Tool details | `tools/definition` with JSON Schema | `agentd <command> --help` |
| Examples | JSON Schema in tool definition | Examples in docs + JSON files |

---

## Client Compatibility

| Client | MCP Support | CLI Support | Recommended |
|--------|-------------|-------------|-------------|
| Claude Code | ✅ Full support | ✅ Full support | Use MCP (native) |
| Gemini CLI | ❌ Connection issues | ✅ Works reliably | Use CLI |
| Codex | ❌ MCP client unavailable | ✅ Works reliably | Use CLI |
| Custom Python/Node | ✅ Via MCP SDK | ✅ Via subprocess | Choose based on needs |

---

## Architecture Benefits

Both MCP and CLI interfaces share the same **service layer**, providing:

1. **Zero Code Duplication**: Business logic written once
2. **Consistency**: Same behavior across interfaces
3. **Easy Testing**: Pure functions in service layer
4. **Future Extensibility**: Can add HTTP API without duplication

```
┌─────────────┐         ┌─────────────┐
│  MCP Tools  │         │ CLI Commands│
│ (JSON-RPC)  │         │ (clap args) │
└──────┬──────┘         └──────┬──────┘
       │                       │
       │   ┌───────────────────┘
       │   │
       ▼   ▼
   ┌──────────────┐
   │Service Layer │ ← Single source of truth
   │(Rust structs)│
   └──────┬───────┘
          │
          ▼
   ┌──────────────┐
   │  Core Logic  │ ← Validators, parsers, models
   └──────────────┘
```

---

## Usage Recommendations

### For LLMs (Gemini, Codex)

**Use CLI commands** due to MCP client compatibility issues:

1. Generate JSON from templates
2. Write to temporary file
3. Execute CLI command with `--json-file`
4. Parse output to verify success

### For Claude Code

**Use MCP tools** (native integration):

- Automatic tool discovery
- Structured JSON-RPC protocol
- Better error handling
- Cleaner integration

### For Human Users

**Use existing workflow commands**:

```bash
agentd plan <id> "<description>"   # Planning workflow
agentd implement <id>               # Implementation workflow
agentd archive <id>                 # Archive workflow
```

Use CLI utility commands only when:
- Debugging specific issues
- Integrating with scripts
- Working around MCP issues

---

## Future Enhancements

### Potential CLI Additions (Not Planned)

- Interactive mode for creation commands
  - `agentd proposal create --interactive`
  - Prompts for each field with validation

- Hybrid flag-based input
  - `agentd proposal create <id> --summary "..." --why "..."`
  - Alternative to JSON files for power users

- HTTP API
  - REST alternative to MCP/CLI
  - Could share the same service layer

### Planned Improvements

- Auto-completion for CLI commands
- Shell integration (bash, zsh, fish)
- Progress indicators for long operations
- Better error messages with suggestions

---

## Getting Help

- **CLI help**: `agentd <command> --help`
- **JSON examples**: See `agentd/specs/cli-guide/examples/`
- **MCP tools**: See `src/mcp/tools/` documentation
- **Issues**: https://github.com/your-repo/agentd/issues
