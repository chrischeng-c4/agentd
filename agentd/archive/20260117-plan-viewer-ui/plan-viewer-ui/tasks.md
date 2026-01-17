# Tasks

## 1. Data Layer

- [ ] 1.1 Create Annotation data models
  - File: `src/models/annotation.rs` (CREATE)
  - Spec: `specs/annotations.md#data-model`
  - Do: Implement `Annotation` and `AnnotationStore` structs with Serde support.
    - `AnnotationStore` must include `change_id` field and `annotations` list.
    - Use `uuid` crate to generate V4 UUIDs for new annotations.
  - Depends: none

- [ ] 1.2 Update models index
  - File: `src/models/mod.rs` (MODIFY)
  - Spec: `specs/annotations.md#data-model`
  - Do: Register `annotation` module.
  - Depends: 1.1

- [ ] 1.3 Add dependencies to Cargo.toml
  - File: `Cargo.toml` (MODIFY)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Add `wry`, `tao`, `include_dir` dependencies behind a `ui` feature flag. Add `uuid` with features `v4` and `serde`. Keep `tempfile` as a dev-dependency but also add it to dependencies (ungated) for atomic write usage in production code. Define `ui` feature in `[features]`.
  - Depends: none

## 2. Logic Layer

- [ ] 2.1 Implement Markdown and YAML rendering
  - File: `src/ui/viewer/render.rs` (CREATE)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Create function to convert Markdown files to HTML using `pulldown-cmark`. Enable GFM options (tables, strikethrough, tasklists). Implement custom heading ID injection (slugification). Implement syntax highlighting logic for code blocks and YAML files.
  - Depends: 1.3

- [ ] 2.2 Implement Viewer state and file loading
  - File: `src/ui/viewer/manager.rs` (CREATE)
  - Spec: `specs/plan-viewer.md#interfaces`
  - Do: Implement logic to read plan files from a `change_id`. Implement `load_file` method that reads content (returning placeholder HTML if missing) AND loads `annotations.json`. Validate paths to prevent traversal.
    - If `annotations.json` parse fails: log warning, rename file to `annotations.json.bak`, and return empty store.
  - Depends: 1.1, 2.1

- [ ] 2.3 Implement IPC Message Handler
  - File: `src/ui/viewer/ipc.rs` (CREATE)
  - Spec: `specs/annotations.md#flow`
  - Do: Handle messages from webview for `load_file`, `save_annotation`, `resolve_annotation`.
    - For `save_annotation`, populate `author` by checking `git config user.name`, then OS user/env, falling back to "unknown".
    - Populate `created_at` with current UTC timestamp.
    - Use atomic writes (write to temp file then rename) for persistence. Return error responses for failures.
  - Depends: 1.1, 2.2

- [ ] 2.4 Create Window Launcher
  - File: `src/ui/viewer/mod.rs` (CREATE)
  - Spec: `specs/plan-viewer.md#interfaces`
  - Do: Implement `start_viewer` using `wry` and `tao`. Implement `with_custom_protocol` to serve bundled assets. Ensure it runs on the main thread (blocking). Guard with `#[cfg(feature = "ui")]`.
  - Depends: 2.3

## 3. Integration

- [ ] 3.1 Register View Command
  - File: `src/cli/mod.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Add `pub mod view;` guarded by `#[cfg(feature = "ui")]`.
  - Depends: none

- [ ] 3.2 Implement View CLI Command
  - File: `src/cli/view.rs` (CREATE)
  - Spec: `specs/plan-viewer.md#acceptance-criteria`
  - Do: Check if `change_id` exists; if not, print "Change '[id]' not found" and exit with status code 1. Resolve path and call `start_viewer`. Guard with `#[cfg(feature = "ui")]`.
  - Depends: 2.4, 3.1

- [ ] 3.3 Register Viewer Module
  - File: `src/ui/mod.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Add `pub mod viewer;` guarded by `#[cfg(feature = "ui")]`.
  - Depends: 2.4

- [ ] 3.4 Refactor Main for Conditional Runtime
  - File: `src/main.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Remove `#[tokio::main]`. Parse CLI args. Match command:
    - If `View` (and `ui` feature): run `agentd::cli::view::run` directly on main thread.
    - Else: build `tokio::runtime::Runtime`, block on async handler. Ensure auto-upgrade check runs in async context.
  - Depends: 3.2

- [ ] 3.5 Auto-open Viewer in Proposal Workflow
  - File: `src/cli/proposal.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#acceptance-criteria`
  - Do: In `proposal.rs`, when phase updates to `CHALLENGED` (signaled by `ChallengeVerdict::Approved`):
    - If `#[cfg(feature = "ui")]`: spawn a detached `agentd view` process.
    - If `#[cfg(not(feature = "ui"))]`: log "UI feature disabled. View plan manually at: [path]".
  - Depends: 3.4

## 4. UI Assets

- [ ] 4.1 Bundle UI Assets
  - File: `src/ui/viewer/assets.rs` (CREATE)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Use `include_dir` to bundle `index.html`, `app.js`, `mermaid.min.js`, `highlight.min.js`, and CSS files into the binary.
  - Depends: 1.3

- [ ] 4.2 Create Webview Template
  - File: `src/ui/viewer/assets/index.html` (CREATE)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Create HTML template with sidebar, content area, and annotation sidebar. Link to bundled JS/CSS using custom protocol URL.
  - Depends: 4.1

- [ ] 4.3 Create Webview Scripts
  - File: `src/ui/viewer/assets/app.js` (CREATE)
  - Spec: `specs/annotations.md#requirements`
  - Do: Implement JS logic for file switching (handling "File not found"), rendering mermaid, syntax highlighting, and handling annotation UI interactions (including error toasts).
  - Depends: 4.2

## 5. Review Actions

- [ ] 5.1 Implement review action IPC handlers
  - File: `src/ui/viewer/ipc.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#r8-review-actions`
  - Do: Add handlers for `approve_review`, `request_changes`, and `submit_comments` IPC messages.
    - `approve_review`: Update `STATE.yaml` phase to `complete`, return success, signal window close.
    - `request_changes`: Save annotations, optionally update phase, return summary.
    - `submit_comments`: Save annotations only, return success.
  - Depends: 2.3

- [ ] 5.2 Add review action buttons to UI
  - File: `src/ui/viewer/assets/index.html` (MODIFY)
  - Spec: `specs/plan-viewer.md#r8-review-actions`
  - Do: Add a toolbar/footer with three buttons: "Approve", "Request Changes", "Submit Comments". Style prominently.
  - Depends: 4.2

- [ ] 5.3 Implement review action JS handlers
  - File: `src/ui/viewer/assets/app.js` (MODIFY)
  - Spec: `specs/plan-viewer.md#r8-review-actions`
  - Do: Wire button clicks to IPC calls. Handle responses (show success/error toast, close window on approve).
  - Depends: 4.3, 5.1

- [ ] 5.4 Add STATE.yaml update logic
  - File: `src/ui/viewer/manager.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#r8-review-actions`
  - Do: Implement `update_phase(change_id, new_phase)` function to modify `STATE.yaml`. Use atomic write.
  - Depends: 2.2

## 6. Testing & Documentation

- [ ] 6.1 Unit tests for Annotation models
  - File: `src/models/annotation.rs` (MODIFY)
  - Spec: `specs/annotations.md#data-model`
  - Do: Test serialization, UUID generation, and metadata population.
  - Depends: 1.1

- [ ] 6.2 Unit test for slug generation
  - File: `src/ui/viewer/render.rs` (MODIFY)
  - Spec: `specs/annotations.md#requirements`
  - Do: Verify slugification logic. Guard with `#[cfg(feature = "ui")]`.
  - Depends: 2.1

- [ ] 6.3 Integration test for HTML rendering
  - File: `tests/viewer_test.rs` (CREATE)
  - Spec: `specs/plan-viewer.md#requirements`
  - Do: Verify HTML generation includes correct IDs and code blocks. Guard with `#[cfg(feature = "ui")]`.
  - Depends: 2.1

- [ ] 6.4 Unit tests for AnnotationStore resilience
  - File: `src/models/annotation.rs` (MODIFY)
  - Spec: `specs/annotations.md#r2-persistence`
  - Do: Test loading from malformed JSON (verify rename to .bak and empty result) and file permission errors.
  - Depends: 1.1

- [ ] 6.5 Unit tests for path traversal
  - File: `src/ui/viewer/manager.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#r7-error-handling`
  - Do: Test that requests for `../secret.txt` are rejected.
  - Depends: 2.2

- [ ] 6.6 Unit tests for review actions
  - File: `src/ui/viewer/ipc.rs` (MODIFY)
  - Spec: `specs/plan-viewer.md#r8-review-actions`
  - Do: Test approve_review updates STATE.yaml, request_changes saves annotations, submit_comments doesn't change phase.
  - Depends: 5.1

- [ ] 6.7 Update README
  - File: `README.md` (MODIFY)
  - Spec: `specs/plan-viewer.md#overview`
  - Do: Add documentation for `agentd view` command and review action buttons. Note that it requires `ui` feature.
  - Depends: 3.2, 5.2
