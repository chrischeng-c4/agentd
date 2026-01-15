# Change: template-extraction

## Summary
Extract hardcoded skeleton templates from `src/context.rs` into external files within `templates/skeletons/` (in the Agentd source) and fix `scripts_dir` to properly resolve relative paths at runtime. Implement a "default with override" strategy for templates, allowing users to customize them by placing files in their project.

## Why
Currently, skeleton templates (for proposals, challenges, reviews) are hardcoded strings in Rust code. This makes them difficult to maintain. Additionally, `scripts_dir` handling needs to be robust and consistently relative to the project root. Users also lack a way to customize these templates without recompiling Agentd.

## What Changes
- Create `templates/skeletons/` directory in the Agentd source repository.
- Extract `proposal.md`, `tasks.md`, `specs/_skeleton.md` (as `spec.md`), `CHALLENGE.md` (as `challenge.md`), and `REVIEW.md` (as `review.md`) into `templates/skeletons/`.
- Update templates to use `{{variable}}` syntax (e.g., `{{change_id}}`) instead of Rust's `{variable}` format.
- Update `src/context.rs` to:
  1.  Look for custom templates in `<Project Root>/agentd/templates/`.
  2.  Fall back to embedded defaults (loaded from `templates/skeletons/` at compile time via `include_str!`).
- Update `scripts_dir` logic to resolve relative paths against project root at runtime (absolute paths remain unchanged). Default `init` writes relative paths for portability.

## Impact
- Affected specs: `specs/extraction.md`
- Affected code:
  - `src/context.rs`: Replace hardcoded strings with file reading logic + embedded fallback.
  - `templates/skeletons/*.md`: New source template files.
  - `src/models/change.rs` / `src/cli/init.rs`: `scripts_dir` handling.
- Breaking changes: No. Existing projects will use embedded defaults (which match current behavior) until they opt-in to overrides.