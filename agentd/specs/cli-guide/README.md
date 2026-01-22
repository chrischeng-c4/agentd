# Agentd CLI Commands

This directory contains JSON examples for using agentd CLI commands. These commands provide an alternative to the MCP (Model Context Protocol) interface for tools like Gemini and Codex that may have MCP compatibility issues.

## Architecture

Agentd uses a **shared service layer** architecture where both MCP tools and CLI commands call the same business logic:

```
MCP Tools (JSON-RPC) ──┐
                        ├──> Service Layer ──> Core Logic
CLI Commands (clap)  ───┘
```

This ensures consistency and zero code duplication between interfaces.

## Available Commands

### Phase 1: Read-Only Commands

These commands allow you to read and list existing data:

```bash
# Knowledge base operations
agentd knowledge list                      # List all knowledge files
agentd knowledge list 00-architecture      # List files in subdirectory
agentd knowledge read index.md             # Read a knowledge file

# Spec operations
agentd spec list <change-id>               # List all specs for a change

# File operations
agentd file read <change-id> proposal      # Read proposal.md
agentd file read <change-id> tasks         # Read tasks.md
agentd file read <change-id> <spec-name>   # Read a specific spec
```

### Phase 2: Creation Commands

These commands create new proposals, specs, tasks, and reviews using JSON input files:

#### 1. Create Proposal

```bash
agentd proposal create <change-id> --json-file proposal-create.json
```

**JSON Format** (`proposal-create.json`):
```json
{
  "summary": "Brief one-line summary (min 10 chars)",
  "why": "Detailed explanation why this change is needed (min 50 chars)",
  "what_changes": [
    "High-level change 1",
    "High-level change 2"
  ],
  "impact": {
    "scope": "patch|minor|major",
    "affected_files": 5,
    "new_files": 2,
    "affected_specs": ["spec-id-1"],
    "affected_code": ["src/path/"],
    "breaking_changes": "Optional description"
  }
}
```

**Example**: See [examples/proposal-create.json](./examples/proposal-create.json)

#### 2. Add Proposal Review

```bash
agentd proposal review <change-id> --json-file proposal-review.json
```

**JSON Format** (`proposal-review.json`):
```json
{
  "status": "approved|needs_revision|rejected",
  "iteration": 1,
  "reviewer": "codex|human|your-name",
  "content": "## Summary\n\n[Review content in markdown]\n\n## Verdict\n\n[Decision]"
}
```

**Example**: See [examples/proposal-review.json](./examples/proposal-review.json)

#### 3. Create Spec

```bash
agentd spec create <change-id> <spec-id> --json-file spec-create.json
```

**JSON Format** (`spec-create.json`):
```json
{
  "title": "Spec Title",
  "overview": "Overview of what this spec covers (min 50 chars)",
  "requirements": [
    {
      "id": "R1",
      "title": "Requirement Title",
      "description": "Detailed description",
      "priority": "high|medium|low"
    }
  ],
  "scenarios": [
    {
      "name": "Scenario Name",
      "given": "Optional precondition",
      "when": "Action or trigger",
      "then": "Expected outcome"
    }
  ],
  "flow_diagram": "Optional Mermaid diagram code",
  "data_model": {
    "Optional": "JSON Schema"
  }
}
```

**Example**: See [examples/spec-create.json](./examples/spec-create.json)

#### 4. Create Tasks

```bash
agentd tasks create <change-id> --json-file tasks-create.json
```

**JSON Format** (`tasks-create.json`):
```json
{
  "tasks": [
    {
      "layer": "data|logic|integration|testing",
      "number": 1,
      "title": "Task Title",
      "file": {
        "path": "src/path/to/file.rs",
        "action": "CREATE|MODIFY|DELETE"
      },
      "spec_ref": "spec-id:R1",
      "description": "Detailed task description",
      "depends": ["1.1", "2.2"]
    }
  ]
}
```

**Example**: See [examples/tasks-create.json](./examples/tasks-create.json)

### Phase 3: Specialized Commands

These commands support advanced workflows like implementation review, clarifications, and knowledge management:

#### 5. Implementation Commands

**Read All Requirements** (proposal + tasks + all specs):
```bash
agentd implementation read-all <change-id>
```

This command reads and consolidates all requirement files for a change in one call, useful during implementation and review phases.

**List Changed Files** (with git statistics):
```bash
agentd implementation list-files <change-id> [--base-branch main] [--filter src/]
```

Lists all changed files with detailed statistics (additions/deletions). Requires a git repository.

Options:
- `--base-branch`: Branch to compare against (default: `main`)
- `--filter`: Filter files by path pattern (e.g., `src/` or `.rs`)

#### 6. Create Clarifications

```bash
agentd clarifications create <change-id> --json-file clarifications-create.json
```

Creates a structured Q&A document to capture planning decisions and rationales.

**JSON Format** (`clarifications-create.json`):
```json
{
  "questions": [
    {
      "topic": "Short topic label",
      "question": "The question asked",
      "answer": "User's answer",
      "rationale": "Why this choice was made"
    }
  ]
}
```

**Example**: See [examples/clarifications-create.json](./examples/clarifications-create.json)

#### 7. Knowledge Base Operations

**Write Knowledge Document**:
```bash
agentd knowledge write <path> --json-file knowledge-write.json
```

Creates or updates a knowledge document with auto-generated frontmatter.

**JSON Format** (`knowledge-write.json`):
```json
{
  "title": "Document Title",
  "source": "https://source-url.com or description",
  "content": "# Markdown Content\n\nYour content here..."
}
```

**Example**: See [examples/knowledge-write.json](./examples/knowledge-write.json)

**Write Main Spec** (for archive workflow):
```bash
agentd knowledge write-spec <path> --content-file spec-content.md
```

Writes a spec to the main `agentd/specs/` directory, used during archive workflow to preserve specifications.

## Task Layer System

Tasks are organized into layers that define the build order:

1. **Data Layer** (`layer: "data"`) - Models, schemas, database migrations
2. **Logic Layer** (`layer: "logic"`) - Business logic, services, core functionality
3. **Integration Layer** (`layer: "integration"`) - API endpoints, external integrations
4. **Testing Layer** (`layer: "testing"`) - Tests for implemented features

Dependencies between tasks are specified using task IDs (e.g., `"depends": ["1.1", "2.2"]`).

## Workflow Examples

### Complete Workflow: Creating a New Feature

```bash
# === Planning Phase ===

# 1. Create clarifications (optional, but recommended)
agentd clarifications create oauth-auth --json-file clarifications-create.json

# 2. Create proposal
agentd proposal create oauth-auth --json-file proposal-create.json

# 3. Create spec
agentd spec create oauth-auth oauth-spec --json-file spec-create.json

# 4. Create implementation tasks
agentd tasks create oauth-auth --json-file tasks-create.json

# 5. Read created files to verify
agentd file read oauth-auth proposal
agentd file read oauth-auth oauth-spec
agentd file read oauth-auth tasks

# === Implementation Phase ===

# 6. Read all requirements in one call (for LLM context)
agentd implementation read-all oauth-auth

# [Implement the changes...]

# 7. List changed files with statistics
agentd implementation list-files oauth-auth

# 8. List only source files
agentd implementation list-files oauth-auth --filter src/

# === Review Phase ===

# 9. After implementation, add review
agentd proposal review oauth-auth --json-file proposal-review.json

# === Archive Phase ===

# 10. Archive completed change (moves specs to main directory)
agentd merge-change oauth-auth
```

### Quick Reference: Command by Workflow Stage

| Stage | Commands |
|-------|----------|
| **Planning** | `proposal create`, `spec create`, `tasks create`, `clarifications create` |
| **Reading** | `knowledge read/list`, `file read`, `spec list`, `implementation read-all` |
| **Implementation** | `implementation read-all`, `implementation list-files` |
| **Review** | `proposal review`, `implementation list-files` |
| **Archive** | `archive`, `knowledge write-spec` |
| **Documentation** | `knowledge write` |

### LLM Usage Pattern

For LLMs (Gemini, Codex) that cannot use MCP:

1. **Generate JSON first** using the examples as templates
2. **Save JSON to temporary file**
3. **Execute CLI command** with the JSON file
4. **Parse output** to verify success

Example Python code for LLM integration:

```python
import json
import subprocess
import tempfile

def create_proposal(change_id, proposal_data):
    """Create proposal using agentd CLI"""
    # Write JSON to temp file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(proposal_data, f)
        temp_file = f.name

    # Execute CLI command
    result = subprocess.run(
        ['agentd', 'proposal', 'create', change_id, '--json-file', temp_file],
        capture_output=True,
        text=True
    )

    # Clean up and return result
    os.unlink(temp_file)
    return result.stdout if result.returncode == 0 else result.stderr
```

## MCP vs CLI Comparison

| Feature | MCP Tools | CLI Commands |
|---------|-----------|--------------|
| Interface | JSON-RPC over stdio | Command-line arguments + JSON files |
| Input Format | JSON via stdin | JSON files on disk |
| Output Format | JSON-RPC response | Plain text with formatting |
| Tool Discovery | `tools/list` method | `--help` flags |
| Error Handling | JSON-RPC errors | Exit codes + stderr |
| Claude Code | ✅ Supported | ✅ Supported |
| Gemini CLI | ❌ MCP issues | ✅ Supported |
| Codex | ❌ MCP issues | ✅ Supported |

## Validation Notes

- Change IDs must be lowercase alphanumeric with hyphens only (e.g., `oauth-auth`)
- Spec IDs follow the same format as change IDs
- `summary` must be at least 10 characters
- `why` must be at least 50 characters
- `overview` must be at least 50 characters
- At least one requirement and one scenario are required for specs
- At least one task is required for tasks files
- Review `content` must be at least 50 characters

## Troubleshooting

### Command Not Found

Make sure agentd is installed and in your PATH:

```bash
cargo install --path .
# or
./target/debug/agentd --version
```

### JSON Parse Error

Validate your JSON syntax:

```bash
cat proposal.json | jq .
```

### Change Not Found

Ensure the change directory exists:

```bash
ls agentd/changes/
```

### Invalid Field Values

Check validation requirements in the JSON format sections above. Common issues:

- Change ID contains uppercase or special characters
- Missing required fields
- Field values too short (summary, why, overview)
- Invalid enum values (scope, priority, status, layer, action)

## Further Reading

- [Agentd Documentation](../../CLAUDE.md)
- [MCP Tools Reference](../../src/mcp/tools/)
- [Service Layer Architecture](../../src/services/)
