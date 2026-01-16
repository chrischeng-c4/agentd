# Challenge Report: simplify-skills

## Summary
The proposal is coherent and implementable, with clear phase-based workflow goals. The main gaps are code alignment for state transitions and skill template naming, plus a migration concern for removing the `testing` phase.

## Internal Consistency Issues
No internal consistency issues found across `agentd/changes/simplify-skills/proposal.md`, `agentd/changes/simplify-skills/tasks.md`, and `agentd/changes/simplify-skills/specs/workflows.md`.

## Code Alignment Issues
Note: The proposal calls out breaking changes (removing `testing`), so some deviations are intentional.

### Issue: Challenge verdict does not update STATE.yaml phase
- **Severity**: Medium
- **Category**: Conflict
- **Description**: Current challenge flow only records validation metadata; it never sets `StatePhase` based on verdict, so phases stay `proposed` even after APPROVED/REJECTED. This blocks the phase-only workflows in `specs/workflows.md#r2`.
- **Location**: `src/cli/challenge_proposal.rs`, `src/cli/validate_challenge.rs`
- **Note**: Not intentional; the proposal requires phase updates tied to verdict.
- **Recommendation**: After parsing verdict, set `StatePhase::Challenged`, `StatePhase::Proposed`, or `StatePhase::Rejected` and persist with `StateManager`.

### Issue: Archive workflow does not set `archived` phase
- **Severity**: Medium
- **Category**: Conflict
- **Description**: `agentd archive` completes without updating `STATE.yaml` to `archived`, which violates `specs/workflows.md#r5` and leaves phase stale in the moved change directory.
- **Location**: `src/cli/archive.rs`
- **Note**: Not intentional; proposal explicitly requires the phase update.
- **Recommendation**: Load `StateManager` and set `StatePhase::Archived` on success before move/cleanup.

### Issue: `testing` phase still in enums/schema/status
- **Severity**: Low
- **Category**: Conflict
- **Description**: Proposal removes the `testing` phase and adds `rejected`, but current enums and schema still include `testing`, and status displays it. This is expected to change but needs to be updated consistently.
- **Location**: `src/models/frontmatter.rs`, `src/models/change.rs`, `agentd/schemas/state.schema.json`, `src/cli/status.rs`
- **Note**: Intentional breaking change per `agentd/changes/simplify-skills/proposal.md`.
- **Recommendation**: Remove `testing`, add `rejected`, and update status icon/color mapping.

### Issue: Skill template name mismatch (`agentd-fix` vs `agentd-resolve-reviews`)
- **Severity**: Medium
- **Category**: Conflict
- **Description**: Proposal/tasks reference `templates/skills/agentd-resolve-reviews/SKILL.md`, but templates install `agentd-fix` and `src/cli/init.rs` includes `SKILL_FIX`. `.claude/skills` already has `agentd-resolve-reviews`, so the canonical name is inconsistent.
- **Location**: `src/cli/init.rs`, `templates/skills/agentd-fix/SKILL.md`, `.claude/skills/agentd-resolve-reviews/SKILL.md`, `agentd/changes/simplify-skills/tasks.md`
- **Note**: Not called out as an intentional rename; tasks explicitly prefer `agentd-resolve-reviews`.
- **Recommendation**: Decide on one canonical name, add/migrate template(s), and update init skill list accordingly (or add an alias).

## Quality Suggestions
### Issue: Backward-compatibility for existing STATE.yaml files
- **Severity**: Low
- **Category**: Completeness
- **Description**: Removing `testing` without a migration will break deserialization of existing `STATE.yaml` that still use `testing`.
- **Recommendation**: Add a migration or compatibility mapping (e.g., treat `testing` as `implementing`) during load.

### Issue: Documentation updates beyond CLAUDE.md
- **Severity**: Low
- **Category**: Other
- **Description**: `templates/README.md` and skill docs still reference the old granular workflow. This will confuse users after deprecation.
- **Recommendation**: Update user-facing docs to reference `/agentd:plan`, `/agentd:impl`, `/agentd:archive`, and the deprecation notice.

## Verdict
- [x] APPROVED - Ready for implementation
- [ ] NEEDS_REVISION - Address issues above (specify which severity levels)
- [ ] REJECTED - Fundamental problems, needs rethinking

**Next Steps**: Proceed with implementation, ensuring the medium-severity code alignment issues are addressed during development.
