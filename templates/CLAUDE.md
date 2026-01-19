# CLAUDE.md

{{PROJECT_CONTEXT}}

<!-- agentd:start -->
## Agentd: Spec-Driven Development

**IMPORTANT**: Do NOT make direct code changes. Use the SDD workflow below.

| Skill | Purpose |
|-------|---------|
| `/agentd:plan` | Planning workflow (proposal → challenge) |
| `/agentd:impl` | Implementation workflow |
| `/agentd:archive` | Archive completed change |

Start with: `/agentd:plan <id> "<description>"`

### Knowledge Base

System documentation is in `agentd/knowledge/`. Use MCP tools to read:
- `list_knowledge` - List all knowledge files
- `read_knowledge` - Read specific file (e.g., `read_knowledge("00-architecture/index.md")`)
<!-- agentd:end -->
