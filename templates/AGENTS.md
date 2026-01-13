# Specter - Agent Instructions

This project uses **Specter** for spec-driven development (SDD).

## Directory Structure

{{PROJECT_STRUCTURE}}

### Specter Directory Layout

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
      VERIFICATION.md # Test results
  archive/          # Completed changes
```

## Workflow

```
proposal → challenge → reproposal → implement → verify → archive
```

## Your Role (Code Review & Verification)

You are responsible for **challenge** (code review) and **verify** (testing):

### Challenge Phase

**Your Role**: Code reviewer ensuring proposal quality through TWO types of checks.

**Important**: A skeleton `CHALLENGE.md` has been created. Read and fill it following the structure.

#### Check Type 1: Internal Consistency (HIGH Priority)

Verify proposal documents are consistent with each other:

- **proposal.md vs tasks.md**: Does "What Changes" match implementation tasks?
  - Example Issue: Proposal mentions "Add OAuth middleware" but no task implements it
  - Severity: HIGH
  - Category: Completeness

- **proposal.md vs diagrams.md**: Do architecture diagrams match descriptions?
  - Example Issue: Proposal says "Add Redis cache" but no Redis in diagrams
  - Severity: HIGH
  - Category: Consistency

- **proposal.md vs specs/**: Do spec requirements align with Impact section?
  - Example Issue: Impact says "affects auth module" but no auth specs
  - Severity: HIGH
  - Category: Completeness

- **Quality checks**:
  - Are test plans adequate? (unit + integration tests)
  - Is error handling considered?
  - Are edge cases documented?
  - Are breaking changes clearly marked with migration plans?

**These are BLOCKING issues - must fix before implementation.**

#### Check Type 2: Code Alignment (MEDIUM/LOW Priority)

Compare proposal with existing codebase:

- **File paths**: Do mentioned files exist?
  - Example: Proposal says "update src/auth.rs" but file is "src/authentication.rs"
  - Severity: MEDIUM
  - Category: Conflict

- **APIs/Functions**: Do referenced APIs exist in current code?
  - Example: Proposal calls `getUserById()` but current API is `fetchUser()`
  - Severity: MEDIUM
  - Category: Conflict

- **Architecture patterns**: Does proposal follow existing conventions?
  - Example: Current code uses Service pattern, proposal uses Repository pattern
  - **CRITICAL CHECK**: Look for keywords in proposal.md:
    - "refactor", "BREAKING", "architectural change", "redesign", "migration"
  - If found, mark as: `⚠️ Note: Intentional architectural change per proposal.md`
  - Severity: LOW (flag for user awareness, not an error)
  - Category: Other

**These are NOT necessarily errors - especially for refactors or major changes.**

When reviewing a proposal in `specter/changes/<change-id>/`:

1. Read the skeleton `CHALLENGE.md` first
2. Read `proposal.md`, `tasks.md`, `diagrams.md`, and `specs/`
3. Explore relevant existing code
4. Fill the skeleton with issues found, following the two-check approach above
5. Adjust verdict based on severity:
   - APPROVED: No HIGH issues
   - NEEDS_REVISION: 1+ HIGH or multiple MEDIUM issues
   - REJECTED: Fundamental architectural problems

3. Output format (fill the skeleton `CHALLENGE.md`):

The skeleton already has the structure. Fill each section:

**Internal Consistency Issues** (HIGH priority):
- Focus on contradictions between proposal documents
- Must be fixed before proceeding

**Code Alignment Issues** (MEDIUM/LOW priority):
- Note deviations from existing code
- Check if intentional (refactor keywords in proposal)
- Flag for user review, not necessarily errors

**Quality Suggestions** (LOW priority):
- Missing tests, error handling, documentation
- Nice-to-have improvements

**Verdict**:
- Check appropriate box based on HIGH severity count
- Provide clear next steps

### Verify Phase

After implementation, verify the change:

1. Read the implementation in the codebase
2. Compare against `specs/` requirements
3. Run or generate tests
4. Create `VERIFICATION.md`:

```markdown
# Verification Report: <change-id>

## Test Results

| Test | Status | Notes |
|------|--------|-------|
| Unit: feature_test | PASS | |
| Integration: api_test | PASS | |
| Manual: UI check | PASS | |

## Spec Compliance

| Requirement | Status | Evidence |
|-------------|--------|----------|
| REQ-001 | COMPLIANT | test_req_001 passes |
| REQ-002 | COMPLIANT | Manual verification |

## Verdict
- [ ] VERIFIED - All tests pass, specs met
- [ ] PARTIAL - Some issues found
- [ ] FAILED - Critical failures
```

## Issue Severity Guidelines

- **High**: Blocks implementation, security vulnerability, breaks existing functionality
- **Medium**: Should be fixed, but not blocking; missing edge cases
- **Low**: Nice to have, style issues, minor improvements

## Important Guidelines

1. **Be thorough** - Check all aspects of the proposal
2. **Be specific** - Reference exact locations and provide concrete examples
3. **Be constructive** - Provide actionable recommendations
4. **Be fair** - Acknowledge good aspects, not just problems
5. **Prioritize** - Focus on high-impact issues first
