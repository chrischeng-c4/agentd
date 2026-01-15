# Tasks

## 1. Templates (Source)
- [ ] 1.1 Create `templates/skeletons/` directory
  - File: `templates/skeletons/` (CREATE DIR)
  - Do: Create the directory if it doesn't exist.
- [ ] 1.2 Extract Proposal Template
  - File: `templates/skeletons/proposal.md` (CREATE)
  - Spec: `specs/extraction.md`
  - Do: Create file with content from `create_proposal_skeleton` (proposal part). Convert `{change_id}` to `{{change_id}}`.
- [ ] 1.3 Extract Tasks Template
  - File: `templates/skeletons/tasks.md` (CREATE)
  - Spec: `specs/extraction.md`
  - Do: Create file with content from `create_proposal_skeleton` (tasks part).
- [ ] 1.4 Extract Spec Template
  - File: `templates/skeletons/spec.md` (CREATE)
  - Spec: `specs/extraction.md`
  - Do: Create file with content from `create_proposal_skeleton` (_skeleton part).
- [ ] 1.5 Extract Challenge Template
  - File: `templates/skeletons/challenge.md` (CREATE)
  - Spec: `specs/extraction.md`
  - Do: Create file with content from `create_challenge_skeleton`. Convert `{change_id}` to `{{change_id}}`.
- [ ] 1.6 Extract Review Template
  - File: `templates/skeletons/review.md` (CREATE)
  - Spec: `specs/extraction.md`
  - Do: Create file with content from `create_review_skeleton`. Convert `{change_id}` to `{{change_id}}` and `{iteration}` to `{{iteration}}`.

## 2. Logic Layer
- [ ] 2.1 Implement Template Loading & Embedding
  - File: `src/context.rs` (MODIFY)
  - Spec: `specs/extraction.md`
  - Do:
    - Add `include_str!` constants for the 5 templates.
    - Implement `load_template(name, project_root)` helper that checks FS then falls back to constants.
    - Update `create_*_skeleton` functions to use `load_template`.
    - Implement simple replacement for `{{change_id}}` etc.
- [ ] 2.2 Config Path Resolution
  - File: `src/models/change.rs` (MODIFY)
  - Spec: `specs/extraction.md`
  - Do: Add `resolve_scripts_dir(&self, project_root: &Path) -> PathBuf` to `AgentdConfig`.
- [ ] 2.3 Update ScriptRunner Usage
  - File: `src/cli/*.rs` (MODIFY)
  - Spec: `specs/extraction.md`
  - Do: Update all calls to `ScriptRunner::new` to pass `config.resolve_scripts_dir(&project_root)`.
  - Files: `src/cli/fix.rs`, `src/cli/archive.rs`, `src/cli/challenge_proposal.rs`, `src/cli/implement.rs`, `src/cli/resolve_reviews.rs`, `src/cli/reproposal.rs`, `src/cli/review.rs`, `src/cli/proposal.rs`.

## 3. Integration
- [ ] 3.1 Verify Init Behavior
  - File: `src/cli/init.rs` (MODIFY)
  - Do: Ensure `init` writes relative `scripts_dir` (e.g., `"agentd/scripts"`) for portability.

## 4. Testing
- [ ] 4.1 Test Template Loading
  - File: `src/context.rs` (MODIFY - add tests)
  - Spec: `specs/extraction.md#acceptance-criteria`
  - Do: Add unit tests for `load_template` - override vs embedded, placeholder replacement.
  - Depends: 2.1
- [ ] 4.2 Test Scripts Dir Resolution
  - File: `src/models/change.rs` (MODIFY - add tests)
  - Spec: `specs/extraction.md#acceptance-criteria`
  - Do: Add unit tests for `resolve_scripts_dir` - relative vs absolute paths.
  - Depends: 2.2
