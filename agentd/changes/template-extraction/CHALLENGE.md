# Challenge Report: template-extraction

## Summary
The proposal is coherent and matches the current code structure. No internal inconsistencies found. A couple of implementation details need clarification in code alignment (project root derivation for template overrides, and init writing relative scripts_dir), but these are straightforward.

## Internal Consistency Issues
No HIGH-severity inconsistencies found across `proposal.md`, `tasks.md`, and `specs/extraction.md`.

## Code Alignment Issues
### Issue: init writes absolute scripts_dir today
- **Severity**: Medium
- **Category**: Conflict
- **Description**: `init` currently persists an absolute scripts path (`agentd_dir.join("scripts")`) into `config.toml`, while the proposal requires a relative default (e.g., `agentd/scripts`) and runtime resolution.
- **Location**: `src/cli/init.rs`
- **Note**: Intentional change per proposal; make sure this gets updated.
- **Recommendation**: Set `config.scripts_dir` to `PathBuf::from("agentd/scripts")` in init, and rely on `AgentdConfig::resolve_scripts_dir` when constructing `ScriptRunner`.

### Issue: Template override lookup needs project_root source
- **Severity**: Medium
- **Category**: Conflict
- **Description**: The spec’s `load_template(name, project_root)` requires a project root, but current `create_*_skeleton` APIs only receive `change_dir` and `change_id`. There’s no explicit path to the project root in those functions today.
- **Location**: `src/context.rs`, call sites in `src/cli/proposal.rs`, `src/cli/challenge_proposal.rs`, `src/cli/review.rs`, `src/cli/implement.rs`
- **Note**: This is an implementation gap to resolve before coding.
- **Recommendation**: Either derive `project_root` from `change_dir` (walk up `agentd/changes/<id>`) or update function signatures and call sites to pass `project_root` explicitly.

## Quality Suggestions
### Issue: Make template overrides discoverable
- **Severity**: Low
- **Category**: Completeness
- **Description**: The proposal adds `<project_root>/agentd/templates/` overrides, but there is no user-facing hint or scaffold in init.
- **Recommendation**: Add a short note in init output or `agentd/project.md`, or create `agentd/templates/README.md` describing override behavior and filenames.

### Issue: Distinguish unreadable override from missing
- **Severity**: Low
- **Category**: Other
- **Description**: `load_template` should not silently fall back when an override exists but is unreadable (permissions, parse errors).
- **Recommendation**: On read errors other than NotFound, surface an actionable error with file path context.

## Verdict
- [x] APPROVED - Ready for implementation
- [ ] NEEDS_REVISION - Address issues above (specify which severity levels)
- [ ] REJECTED - Fundamental problems, needs rethinking

**Next Steps**: Implement the changes, ensuring `project_root` is available for template overrides and init writes a relative `scripts_dir`.
