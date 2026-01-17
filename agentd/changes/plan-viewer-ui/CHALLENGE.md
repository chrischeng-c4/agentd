# Challenge Report: plan-viewer-ui

## Summary
Feasible overall, but there are a few correctness gaps around file safety, asset rendering, and task/spec mismatches that should be resolved before implementation.

## Internal Consistency Issues
### Issue: Corrupt annotations handling is split between tasks and tests
- **Severity**: High
- **Category**: Consistency
- **Description**: The spec requires renaming malformed `annotations.json` to `.bak` and returning an empty store. Tasks 2.2 place this logic in `viewer/manager.rs`, but task 5.4 expects this behavior in `src/models/annotation.rs` tests. The ownership of this logic is unclear.
- **Location**: `agentd/changes/plan-viewer-ui/tasks.md` (2.2, 5.4), `agentd/changes/plan-viewer-ui/specs/annotations.md#R2`
- **Recommendation**: Decide whether corruption handling belongs in the model or manager, then align tasks/tests accordingly (e.g., move tests to `viewer/manager.rs` or move logic into `AnnotationStore::load`).

## Code Alignment Issues
### Issue: Path traversal protection not specified for save/update paths
- **Severity**: High
- **Category**: Conflict
- **Description**: The spec requires file operations be restricted to the change directory. Tasks only mention validating paths in `load_file`, but `save_annotation` and `resolve_annotation` also accept file inputs and can write to disk.
- **Location**: `agentd/changes/plan-viewer-ui/tasks.md` (2.3), `agentd/changes/plan-viewer-ui/specs/annotations.md#R4`
- **Recommendation**: Validate and normalize `file` for all IPC write operations (restrict to `proposal.md`, `CHALLENGE.md`, `STATE.yaml`), and reject any path separators or traversal attempts.

### Issue: Auto-open viewer spawn likely fails in dev builds
- **Severity**: Medium
- **Category**: Conflict
- **Description**: The proposal implies spawning `agentd view` as a detached process. In the current codebase, spawning the current binary uses `std::env::current_exe()` (see `init.rs`). Relying on `agentd` being on PATH can fail in `cargo run` or local builds.
- **Location**: `agentd/changes/plan-viewer-ui/tasks.md` (3.5)
- **Note**: This is not documented in the proposal.
- **Recommendation**: Use `current_exe()` and set `current_dir(project_root)` when spawning the viewer to ensure it resolves the correct change path.

## Quality Suggestions
### Issue: Mermaid rendering path not fully specified
- **Severity**: Low
- **Category**: Completeness
- **Description**: The tasks mention enabling Mermaid in JS, but do not specify how markdown code fences are converted to Mermaid nodes. `mermaid.run` expects `.mermaid` containers, not `<pre><code class="language-mermaid">`.
- **Recommendation**: In the renderer or JS, transform `language-mermaid` code blocks into `<div class="mermaid">...</div>` before calling `mermaid.run`.

### Issue: Duplicate heading slugs can break annotations
- **Severity**: Low
- **Category**: Other
- **Description**: Slugification is deterministic but does not account for repeated headings, which can produce duplicate IDs and mis-associate annotations.
- **Recommendation**: Add a collision suffix (`-2`, `-3`) during rendering to keep IDs unique and stable.

### Issue: Atomic write behavior on Windows not called out
- **Severity**: Low
- **Category**: Other
- **Description**: `tempfile` persistence can fail when overwriting existing files on Windows if not handled explicitly.
- **Recommendation**: Use a replace-safe strategy (write temp file in same dir, `persist` after removing existing target, or custom rename logic) and add a note in implementation.

## Verdict
- [ ] APPROVED - Ready for implementation
- [x] NEEDS_REVISION - Address issues above (specify which severity levels)
- [ ] REJECTED - Fundamental problems, needs rethinking

**Next Steps**: Clarify ownership of corruption handling/tests and update IPC write validation; then refine the spawn strategy and Mermaid rendering plan.
