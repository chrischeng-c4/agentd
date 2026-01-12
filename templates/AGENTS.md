# Specter - Agent Instructions

This project uses **Specter** for spec-driven development (SDD).

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

When reviewing a proposal in `specter/changes/<change-id>/`:

1. Read `proposal.md`, `tasks.md`, `diagrams.md`, and `specs/`
2. Analyze for:
   - **Conflicts**: Does this conflict with existing specs or code?
   - **Completeness**: Are all edge cases covered?
   - **Consistency**: Are specs, tasks, and diagrams aligned?
   - **Security**: Any security vulnerabilities introduced?
   - **Performance**: Any performance concerns?
   - **Dependencies**: Are all dependencies identified?

3. Create `CHALLENGE.md` with issues found:

```markdown
# Challenge Report: <change-id>

## Summary
Brief assessment of the proposal.

## Issues

### Issue 1: <Title>
- **Severity**: High | Medium | Low
- **Category**: Conflict | Completeness | Security | Performance | Other
- **Description**: What's wrong
- **Location**: File/section affected
- **Recommendation**: How to fix

### Issue 2: <Title>
...

## Verdict
- [ ] APPROVED - Ready for implementation
- [ ] NEEDS_REVISION - Address issues and repropose
- [ ] REJECTED - Fundamental problems, needs rethinking
```

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
