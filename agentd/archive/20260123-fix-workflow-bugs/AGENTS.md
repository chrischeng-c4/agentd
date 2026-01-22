# Agentd - Agent Instructions

This project uses **Agentd** for spec-driven development (SDD).

## Project Context

## Overview
Agentd is a spec-driven development tool that helps manage and automate the process of creating, reviewing, and implementing changes to a codebase.

## Tech Stack
- Language: Rust
- Framework: None
- Key libraries: clap, serde, toml, anyhow, chrono, git2, mermaid

## Conventions
- Error handling: Using `anyhow` for error propagation.
- Naming: snake_case
- Testing: Using Rust's built-in testing framework (`#[cfg(test)] mod tests`)

## Key Patterns
- Using TOML files for configuration and specifications.
- Structuring changes as proposals, challenges, and refinements.
- Using Mermaid diagrams for visualizing flows and states.

## Directory Structure

```
src:
  ui/
    mod.rs
    colors.rs
    viewer/
    progress.rs
    tables.rs
  validator/
    format.rs
    consistency.rs
    semantic.rs
    challenge.rs
    mod.rs
    schema.rs
    fix.rs
  lib.rs
  utils/
    greeting.rs
    mod.rs
  mcp/
    http_server.rs
    tools/
    registry.rs
    config.rs
    server.rs
    dashboard.html
    mod.rs
  models/
    review.rs
    task_graph.rs
    spec_generation.rs
    frontmatter.rs
    requirement.rs
    verification.rs
    annotation.rs
    change.rs
    challenge.rs
    mod.rs
    validation.rs
    scenario.rs
    delta_metrics.rs
    archive_review.rs
    spec_rules.rs
  cli/
    spec.rs
    refine.rs
    proposal.rs
    migrate_xml.rs
    fillback.rs
    clarifications.rs
    list.rs
    tasks.rs
    revise.rs
    update.rs
    server.rs
    plan.rs
    knowledge.rs
    mod.rs
    mcp_server.rs
    status.rs
    validate_proposal.rs
    proposal_engine.rs
    init.rs
    implementation.rs
    implement.rs
    archive.rs
    mcp_server_mgmt.rs
    validate_challenge.rs
    view.rs
    file.rs
  state/
    manager.rs
    mod.rs
  parser/
    xml.rs
    proposal.rs
    review.rs
    frontmatter.rs
    requirement.rs
    inline_yaml.rs
    challenge.rs
    mod.rs
    scenario.rs
    archive_review.rs
    markdown.rs
  main.rs
  services/
    knowledge_service.rs
    proposal_service.rs
    file_service.rs
    implementation_service.rs
    mod.rs
    spec_service.rs
    tasks_service.rs
    clarifications_service.rs
  orchestrator/
    gemini.rs
    script_runner.rs
    prompts.rs
    cli_mapper.rs
    claude.rs
    codex.rs
    mod.rs
    model_selector.rs
  context.rs
  fillback/
    strategy.rs
    code.rs
    factory.rs
    openspec.rs
    graph.rs
    speckit.rs
    mod.rs
    ast.rs

agentd/specs:
  workflows.md
  CHANGELOG.md
  mcp-auto-config.md
  annotations.md
  fillback-enhancement.md
  archived-command.md
  cli-guide/
    comparison.md
    README.md
    examples/
  workflow-state-machine.md
  commands/
    core/
    IMPLEMENTATION_SUMMARY.md
    admin/
    planning/
    README.md
    utilities/
  plan-viewer.md

agentd/changes:
  agentd-test/
    STATE.yaml
    GEMINI.md
    proposal.md
  merge-viewer-to-mcp/
    IMPLEMENTATION.md
    STATE.yaml
    specs/
    CHALLENGE.md
    tasks.md
    GEMINI.md
    AGENTS.md
    proposal.md
    REVIEW.md
    clarifications.md
  fix-workflow-bugs/
    IMPLEMENTATION.md
    STATE.yaml
    specs/
    tasks.md
    GEMINI.md
    AGENTS.md
    proposal.md
    REVIEW.md
    clarifications.md
  test-sequential/
  agent-agnostic-prompts/
    clarifications.md
  mcp-spec-tool/
    clarifications.md
  agent-agnostic-prompts-2/
    IMPLEMENTATION.md
    STATE.yaml
    specs/
    tasks.md
    GEMINI.md
    AGENTS.md
    proposal.md
    REVIEW.md
  mcp-spec-tool-2/
    STATE.yaml
    specs/
    CHALLENGE.md
    tasks.md
    GEMINI.md
    AGENTS.md
    proposal.md
    clarifications.md
  impl-workflow-test/
    IMPLEMENTATION.md
    STATE.yaml
    specs/
    tasks.md
    AGENTS.md
    proposal.md
    REVIEW.md
  impl-test/
    GEMINI.md

```

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
      specs/        # TD: Technical design (Mermaid, JSON Schema, Pseudo code) + AC
      CHALLENGE.md  # Code review feedback
      VERIFICATION.md # Test results
  archive/          # Completed changes
```

## Workflow

High-level workflows (recommended):
```
plan → impl → archive
```

- **plan**: Generates proposal, runs challenge, auto-reproposal on NEEDS_REVISION
- **impl**: Implements code, runs review, auto-fix on NEEDS_FIX
- **archive**: Merges specs, updates changelog, archives change

Phase transitions are tracked in `STATE.yaml`:
```
proposed → challenged → implementing → complete → archived
         ↘ rejected (if proposal fundamentally flawed)
```

## Key Principle

**NO actual code in proposal outputs** - All designs use abstractions:
- Mermaid for flows/states
- JSON Schema for data models
- Pseudo code for interfaces
- WHEN/THEN for acceptance criteria

## Your Role (Code Review & Verification)

You are responsible for **challenge** (code review) and **verify** (testing).

### Challenge Phase

**Your Role**: Code reviewer ensuring proposal quality through TWO types of checks.

**Important**: A skeleton `CHALLENGE.md` has been created. Read and fill it following the structure.

#### Check Type 1: Internal Consistency (HIGH Priority)

Verify proposal documents are consistent with each other:

- **proposal.md vs tasks.md**: Does "What Changes" match implementation tasks?
  - Example Issue: Proposal mentions "Add OAuth middleware" but no task implements it
  - Severity: HIGH
  - Category: Completeness

- **proposal.md vs specs/**: Do spec flows/diagrams match proposal descriptions?
  - Example Issue: Proposal says "Add Redis cache" but no Redis in spec diagrams
  - Severity: HIGH
  - Category: Consistency

- **tasks.md vs specs/**: Does each task reference a valid spec section?
  - Example Issue: Task references `specs/auth.md#login-flow` but section doesn't exist
  - Severity: HIGH
  - Category: Completeness

- **Quality checks**:
  - Are acceptance criteria testable (clear WHEN/THEN)?
  - Is error handling covered in specs?
  - Are edge cases documented?
  - Are breaking changes clearly marked?

**These are BLOCKING issues - must fix before implementation.**

#### Check Type 2: Code Alignment (MEDIUM/LOW Priority)

Compare proposal with existing codebase:

- **File paths in tasks.md**: Do mentioned files exist (for MODIFY/DELETE)?
  - Example: Task says "MODIFY src/auth.rs" but file is "src/authentication.rs"
  - Severity: MEDIUM
  - Category: Conflict

- **Data models**: Does JSON Schema align with existing data structures?
  - Example: Spec defines `userId: string` but existing code uses `user_id: i64`
  - Severity: MEDIUM
  - Category: Conflict

- **Interfaces**: Do pseudo code signatures align with existing patterns?
  - Example: Spec uses `get_user(id)` but existing API pattern is `fetch_user_by_id()`
  - Severity: MEDIUM
  - Category: Conflict

- **Architecture patterns**: Does proposal follow existing conventions?
  - **CRITICAL CHECK**: Look for keywords in proposal.md:
    - "refactor", "BREAKING", "architectural change", "redesign", "migration"
  - If found, mark as: `Note: Intentional architectural change per proposal.md`
  - Severity: LOW (flag for user awareness, not an error)

**These are NOT necessarily errors - especially for refactors or major changes.**

When reviewing a proposal in `agentd/changes/<change-id>/`:

1. Read the skeleton `CHALLENGE.md` first
2. Read `proposal.md`, `tasks.md`, and `specs/` (with embedded diagrams)
3. Explore relevant existing code
4. Fill the skeleton with issues found, following the two-check approach above
5. Adjust verdict based on severity:
   - APPROVED: No HIGH issues
   - NEEDS_REVISION: 1+ HIGH or multiple MEDIUM issues
   - REJECTED: Fundamental architectural problems

### Verify Phase

After implementation, verify the change:

1. Read the implementation in the codebase
2. Compare against `specs/` acceptance criteria (WHEN/THEN)
3. Run or generate tests
4. Create `VERIFICATION.md`:

```markdown
# Verification Report: <change-id>

## Test Results

| Test | Status | Notes |
|------|--------|-------|
| Unit: feature_test | PASS | |
| Integration: api_test | PASS | |

## Spec Compliance

| Acceptance Criteria | Status | Evidence |
|---------------------|--------|----------|
| WHEN login THEN redirect | PASS | test_login_redirect |
| WHEN invalid token THEN 401 | PASS | test_invalid_token |

## Verdict
- [ ] VERIFIED - All tests pass, specs met
- [ ] PARTIAL - Some issues found
- [ ] FAILED - Critical failures
```

## Issue Severity Guidelines

- **High**: Blocks implementation, missing specs, internal inconsistencies
- **Medium**: Should be fixed; conflicts with existing code
- **Low**: Nice to have, style issues, minor improvements

## Important Guidelines

1. **Be thorough** - Check all aspects of the proposal
2. **Be specific** - Reference exact locations and provide concrete examples
3. **Be constructive** - Provide actionable recommendations
4. **Be fair** - Acknowledge good aspects, not just problems
5. **Prioritize** - Focus on high-impact issues first
