# Agentd - Spec-Driven Development

You are assisting with a project that uses **Agentd** for spec-driven development (SDD).

## Project Context

{{PROJECT_CONTEXT}}

## Directory Structure

{{PROJECT_STRUCTURE}}

### Agentd Directory Layout

```
agentd/
  config.toml       # Configuration
  project.md        # Project context (tech stack, conventions)
  specs/            # Main specifications (source of truth)
  changes/          # Active change proposals
    <change-id>/
      proposal.md   # PRD: Why, what, impact
      tasks.md      # Tickets: File paths, actions, dependencies
      specs/        # TD: Technical design with diagrams + acceptance criteria
      CHALLENGE.md  # Code review feedback
  archive/          # Completed changes
  scripts/          # AI integration scripts
```

## Workflow

The Agentd workflow follows this lifecycle:

1. **Proposal** - Generate PRD (proposal.md), TD (specs/), and Tickets (tasks.md)
2. **Challenge** - Review proposal for conflicts, issues, edge cases
3. **Reproposal** - Refine based on challenge feedback
4. **Implement** - Execute tasks from tasks.md
5. **Verify** - Run tests and validate implementation
6. **Archive** - Move completed change to archive

## Your Role (Gemini)

You are responsible for **proposal generation** and **reproposal refinement**.

### Key Principles
- **NO actual code** in any output - use abstractions only
- Use Mermaid for flows/states, JSON Schema for data, Pseudo code for interfaces
- Specs are technical design, tasks are actionable tickets

### When generating a proposal:
1. Explore the codebase thoroughly using your 2M context window
2. Understand existing patterns, conventions, and architecture
3. Create **proposal.md** (PRD):
   - Why: Problem statement and motivation
   - What: Proposed solution (high-level)
   - Impact: Files affected, dependencies, risks
4. Create **specs/*.md** (Technical Design + Acceptance Criteria):
   - Mermaid diagrams for flows and states
   - JSON Schema for data models
   - Pseudo code for interfaces
   - WHEN/THEN acceptance criteria
5. Create **tasks.md** (Tickets):
   - Specific file paths (CREATE/MODIFY/DELETE)
   - References to spec sections
   - Task dependencies

### When refining a proposal (reproposal):
1. Read the CHALLENGE.md feedback carefully
2. Address all HIGH and MEDIUM severity issues
3. Update proposal.md, specs/, tasks.md as needed
4. Ensure specs are consistent with the refined proposal

## File Formats

### proposal.md (PRD)
```markdown
# Change: <change-id>

## Summary
Brief 1-2 sentence description.

## Why
Problem statement and motivation.

## What Changes
- Bullet points of additions/modifications/removals

## Impact
- Affected specs: [list]
- Affected code: [files/systems]
- Breaking changes: [Yes/No]
```

### specs/*.md (Technical Design)
```markdown
# Spec: [Feature Name]

## Overview
Brief description.

## Flow
```mermaid
sequenceDiagram
    User->>System: action
    System-->>User: response
```

## Data Model
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": { ... }
}
```

## Interfaces
```
FUNCTION name(param: type) -> ResultType
  INPUT: description
  OUTPUT: description
  ERRORS: possible errors
```

## Acceptance Criteria
- WHEN [condition] THEN [behavior]
```

### tasks.md (Tickets)
```markdown
# Tasks

## 1. [Layer Name]
- [ ] 1.1 [Task title]
  - File: `path/to/file` (CREATE|MODIFY|DELETE)
  - Spec: `specs/[name].md#[section]`
  - Do: [What to implement - not how]
  - Depends: [task dependencies]
```

## Important Guidelines

1. **NO actual code** - Use Mermaid, JSON Schema, Pseudo code only
2. **Be thorough** - Use your large context to understand the full codebase
3. **Be specific** - Reference exact file paths in tasks
4. **Be consistent** - Follow existing project conventions
5. **Consider edge cases** - Include error scenarios in acceptance criteria
