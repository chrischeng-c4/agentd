# Change: plan-viewer-ui

## Summary

Add a standalone UI window viewer for `agentd` plans using `wry`. This viewer provides a rich, interactive interface for reviewing proposals and challenges, featuring native Mermaid diagram rendering, state visualization, and support for human annotations.

## Why

Reviewing AI-generated technical designs in raw Markdown is cumbersome, especially for complex systems with multiple Mermaid diagrams. A dedicated viewer improves design quality by:
- Providing a clearer visualization of architecture and flows.
- Enabling human-in-the-loop feedback through annotations.
- Streamlining the transition from "Proposed" to "Challenged" and finally to "Implementing" phases by making the review process more efficient and thorough.

## What Changes

### CLI
- Add `agentd view <change-id>` command to open the viewer on demand.
- Integrate auto-open logic in `agentd proposal` (and the planning loop) to trigger when a proposal reaches the `challenged` phase.
    - If the `ui` feature is enabled, this spawns a detached viewer process.
    - If the `ui` feature is disabled, this logs a helpful message suggesting the user view the plan manually or enable the feature.

### UI
- Implementation of a native window using `wry` and `tao`.
- Multi-pane interface:
    - Sidebar for navigation between files (`proposal.md`, `CHALLENGE.md`, `STATE.yaml`).
    - Main content area with Markdown rendering and YAML rendering.
    - Right sidebar or overlay for annotations.
- Native Mermaid diagram rendering using bundled `mermaid.js` assets inside the webview.
- Responsive design for different window sizes.

### Logic
- Markdown to HTML conversion engine for plan files with GFM support and stable heading anchors (injected by backend).
- YAML rendering engine for `STATE.yaml` with syntax highlighting.
- Annotation system:
    - Ability to highlight/select sections in the UI.
    - Save comments to a persistent `annotations.json` file in the change directory (using atomic writes).
    - Use UUIDs for unique annotation identification.
    - Automatic population of metadata (author, timestamp) with fallbacks.
- IPC communication between the webview and the *viewer process* (Rust backend) for handling file I/O, window actions, and initial data loading.

## Impact

- Affected specs:
    - `specs/plan-viewer.md` (New)
    - `specs/annotations.md` (New)
- Affected code:
    - `Cargo.toml`: Add `wry`, `tao`, `uuid`, `include_dir`, `tempfile` dependencies behind a `ui` feature flag.
    - `src/main.rs`: Refactor entry point to allow conditional main-thread execution. Add `view` command.
    - `src/cli/mod.rs`: Register new command.
    - `src/cli/view.rs`: (New) Command implementation.
    - `src/cli/proposal.rs`: Add auto-open trigger logic.
    - `src/ui/mod.rs`: Register `viewer` module.
    - `src/ui/viewer/`: (New) UI implementation logic.
    - `README.md`: Update documentation with new command.
- Breaking changes: No (but changes `src/main.rs` entry point behavior).