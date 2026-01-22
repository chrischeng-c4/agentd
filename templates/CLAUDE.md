# CLAUDE.md

{{PROJECT_CONTEXT}}

<!-- agentd:start -->
## Agentd: Spec-Driven Development

**IMPORTANT**: Do NOT make direct code changes. Use the SDD workflow below.

| Skill | Purpose |
|-------|---------|
| `/agentd:plan-change` | Planning workflow (proposal → challenge → auto-reproposal loop) |
| `/agentd:impl-change` | Implementation workflow (implement → review → auto-resolve loop) |
| `/agentd:merge-change` | Archive completed change |

All workflows are **state-aware** and resume automatically from the current phase.

Start with: `/agentd:plan-change <id> "<description>"`

### Knowledge Base

System documentation is in `agentd/knowledge/`. Use CLI commands to read:
```bash
agentd knowledge list                # List all knowledge files
agentd knowledge read <path>         # Read specific file
```

### CLI Commands

The agentd workflows use CLI commands with JSON files for all operations. All commands use the same service layer as the MCP server (when available), ensuring consistent behavior.

**Complete Documentation**:
- **CLI Guide**: See `agentd/specs/cli-guide/README.md`
- **JSON Examples**: See `agentd/specs/cli-guide/examples/`

**Quick Reference**:

**Read Operations** (no JSON needed):
```bash
agentd knowledge list                    # List knowledge files
agentd knowledge read <path>             # Read knowledge file
agentd spec list <change-id>             # List specs
agentd file read <change-id> proposal    # Read proposal.md
agentd implementation read-all <id>      # Read all requirements
agentd implementation list-files <id>    # List changed files
```

**Creation Operations** (using JSON files):
```bash
agentd proposal create <id> --json-file proposal.json
agentd spec create <id> <spec-id> --json-file spec.json
agentd tasks create <id> --json-file tasks.json
agentd proposal review <id> --json-file review.json
agentd clarifications create <id> --json-file clarifications.json
agentd knowledge write <path> --json-file knowledge.json
```

**Phase Summary**:
- **Phase 1 (Read-Only)**: 5 commands - knowledge, spec, file operations
- **Phase 2 (Creation)**: 4 commands - proposal, spec, tasks, review
- **Phase 3 (Specialized)**: 5 commands - implementation, clarifications, knowledge write

**LLM Usage Pattern**:
1. Generate JSON from examples in `agentd/specs/cli-guide/examples/`
2. Write JSON to temporary file
3. Execute CLI command with `--json-file`
4. Parse output to verify success
<!-- agentd:end -->
