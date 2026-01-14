# CLAUDE.md

{{PROJECT_CONTEXT}}

## Agentd Workflow

Use these skills for spec-driven development:

| Skill | Purpose |
|-------|---------|
| `/agentd:proposal` | Generate proposal with Gemini |
| `/agentd:challenge` | Review proposal with Codex |
| `/agentd:reproposal` | Refine based on feedback |
| `/agentd:implement` | Implement the change |
| `/agentd:review` | Run tests and code review |
| `/agentd:fix` | Fix issues from review |
| `/agentd:archive` | Archive completed change |

## File Structure

- `agentd/project.md` - Project context
- `agentd/specs/` - Main specifications
- `agentd/changes/<id>/` - Active changes
  - `proposal.md` - PRD (why, what)
  - `tasks.md` - Implementation tasks
  - `specs/` - Technical design + acceptance criteria
