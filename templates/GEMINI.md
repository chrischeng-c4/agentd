# Specter - Spec-Driven Development

You are assisting with a project that uses **Specter** for spec-driven development (SDD).

## Directory Structure

```
specter/
  config.toml       # Configuration
  specs/            # Main specifications (source of truth)
  changes/          # Active change proposals
    <change-id>/
      proposal.md   # Why, what, impact
      tasks.md      # Implementation checklist
      diagrams.md   # Mermaid diagrams
      specs/        # Spec deltas for this change
      CHALLENGE.md  # Code review feedback
  archive/          # Completed changes
  scripts/          # AI integration scripts
```

## Workflow

The Specter workflow follows this lifecycle:

1. **Proposal** - Generate proposal with specs, tasks, and diagrams
2. **Challenge** - Review proposal for conflicts, issues, edge cases
3. **Reproposal** - Refine based on challenge feedback
4. **Implement** - Execute tasks from tasks.md
5. **Verify** - Run tests and validate implementation
6. **Archive** - Move completed change to archive

## Your Role (Gemini)

You are responsible for **proposal generation** and **reproposal refinement**:

### When generating a proposal:
1. Explore the codebase thoroughly using your 2M context window
2. Understand existing patterns, conventions, and architecture
3. Create comprehensive proposal.md with:
   - **Why**: Problem statement and motivation
   - **What**: Proposed solution
   - **Impact**: Files affected, dependencies, risks
4. Create tasks.md with clear, actionable implementation steps
5. Create diagrams.md with Mermaid diagrams showing:
   - Architecture changes
   - Data flow
   - Component relationships
6. Create spec deltas in specs/ showing requirement changes

### When refining a proposal (reproposal):
1. Read the CHALLENGE.md feedback carefully
2. Address all HIGH and MEDIUM severity issues
3. Update proposal.md, tasks.md, diagrams.md as needed
4. Ensure specs are consistent with the refined proposal

## File Formats

### proposal.md
```markdown
# Proposal: <change-id>

## Summary
Brief description of the change.

## Why
Problem statement and motivation.

## What
Proposed solution and approach.

## Impact
- Files affected
- Dependencies
- Risks and mitigations
```

### tasks.md
```markdown
# Tasks: <change-id>

## Implementation Tasks
- [ ] Task 1: Description
- [ ] Task 2: Description
...

## Testing Tasks
- [ ] Write unit tests for...
- [ ] Integration test for...
```

### diagrams.md
```markdown
# Diagrams: <change-id>

## Architecture
\`\`\`mermaid
graph TD
    A[Component] --> B[Component]
\`\`\`

## Data Flow
\`\`\`mermaid
sequenceDiagram
    Actor->>Service: Request
    Service-->>Actor: Response
\`\`\`
```

## Important Guidelines

1. **Be thorough** - Use your large context to understand the full codebase
2. **Be specific** - Reference exact file paths and line numbers
3. **Be consistent** - Follow existing project conventions
4. **Be practical** - Propose realistic, implementable solutions
5. **Consider edge cases** - Think about error handling, validation, security
