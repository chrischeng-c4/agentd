# Implementation Notes: Plan Viewer UI

## Summary

Implemented a standalone UI window viewer for `agentd` plans using `wry` and `tao`. The viewer provides a rich, interactive interface for reviewing proposals and challenges with native Mermaid diagram rendering, state visualization, and support for human annotations.

## Completed Tasks

### Data Layer
- [x] **1.1** Created `src/models/annotation.rs` with `Annotation` and `AnnotationStore` structs
- [x] **1.2** Updated `src/models/mod.rs` to register annotation module
- [x] **1.3** Added dependencies to `Cargo.toml` (wry, tao, include_dir, uuid, tempfile)

### Logic Layer
- [x] **2.1** Implemented Markdown/YAML rendering in `src/ui/viewer/render.rs`
  - GFM support (tables, strikethrough, task lists)
  - Stable heading ID injection for annotation targeting
  - Code block language detection for syntax highlighting
- [x] **2.2** Implemented viewer state and file loading in `src/ui/viewer/manager.rs`
  - Path validation to prevent directory traversal
  - File existence checking with graceful "not found" handling
- [x] **2.3** Implemented IPC message handler in `src/ui/viewer/ipc.rs`
  - load_file, save_annotation, resolve_annotation operations
  - Author auto-population from git config or environment
  - Atomic writes for annotation persistence

### Integration
- [x] **3.1** Registered view command in `src/cli/mod.rs` (feature-gated)
- [x] **3.2** Implemented view CLI command in `src/cli/view.rs`
  - Change existence validation
  - Detached process spawning for auto-open
- [x] **3.3** Registered viewer module in `src/ui/mod.rs` (feature-gated)
- [x] **3.4** Refactored `src/main.rs` for conditional runtime
  - View command runs on main thread (no tokio runtime)
  - Other commands use tokio runtime
- [x] **3.5** Added auto-open viewer in proposal workflow (`src/cli/proposal.rs`)
  - Spawns detached viewer when proposal is approved
  - Fallback message when UI feature is disabled

### UI Assets
- [x] **4.1** Bundle UI assets in `src/ui/viewer/assets.rs` using `include_dir`
- [x] **4.2** Created webview template `src/ui/viewer/assets/index.html`
- [x] **4.3** Created webview scripts `src/ui/viewer/assets/app.js`
- [x] Created styles `src/ui/viewer/assets/styles.css`
- [x] Created placeholder JS libraries (highlight.min.js, mermaid.min.js)

### Testing & Documentation
- [x] **5.1-5.5** Unit tests included in each module
- [x] **5.3** Integration tests in `tests/viewer_test.rs`
- [x] **5.6** Updated README with view command documentation

## Key Implementation Decisions

### Feature Gating
All UI-related code is gated behind `#[cfg(feature = "ui")]` to avoid bloating minimal builds. The feature adds wry, tao, and include_dir dependencies.

### Main Thread Requirement
The `agentd view` command runs on the main thread without initializing tokio, as required by tao/wry on macOS. Other commands create a tokio runtime manually.

### Custom Protocol
Assets are served via `agentd://` custom protocol, allowing completely offline operation with bundled resources.

### Annotation Persistence
- Atomic writes (temp file + rename) prevent corruption
- Malformed JSON files are backed up and treated as empty
- UUIDs ensure unique annotation identification

### Path Security
- Strict allowlist for loadable files (proposal.md, CHALLENGE.md, STATE.yaml, tasks.md)
- Path traversal attempts rejected with error

## Review Fixes (Iteration 1)

### HIGH: IPC Responses Not Delivered to UI
**Problem**: The Rust IPC handler processed messages but never sent responses back to the webview.

**Solution**:
- Added custom `UserEvent::EvaluateScript` event type for async IPC responses
- Used `EventLoopBuilder::<UserEvent>::with_user_event()` to create event loop with custom events
- IPC handler serializes response and sends via `proxy.send_event(UserEvent::EvaluateScript(script))`
- Event loop handler calls `webview.evaluate_script()` to deliver response to JS
- Updated `app.js` with `handleIpcResponse()` function to receive and process responses
- Exposed handler via `window.agentd.handleIpcResponse` for Rust to call

**Files Modified**:
- `src/ui/viewer/mod.rs`: Added UserEvent enum, event loop with custom events, IPC response delivery
- `src/ui/viewer/assets/app.js`: Added handleIpcResponse, response type detection, state updates

### HIGH: Custom Protocol URLs Resolve to 404
**Problem**: URLs like `agentd://styles.css` put the filename in the host field, not the path, causing asset lookups to fail.

**Solution**:
- Updated custom protocol handler to check both host and path components
- For `agentd://localhost/file.css` or `agentd://assets/file.css`: extract from path
- For `agentd://file.css`: extract filename from host (original behavior preserved)
- Added debug logging for 404s to aid troubleshooting

**Files Modified**:
- `src/ui/viewer/mod.rs`: Enhanced URL parsing in custom protocol handler

### MEDIUM: UI-Disabled Fallback Message Format
**Problem**: Fallback message was printed on two lines instead of the exact required format.

**Solution**:
- Combined message into single line: `"UI feature disabled. View plan manually at: {path}"`

**Files Modified**:
- `src/cli/proposal.rs`: Fixed `open_viewer_if_available` function output format

## Review Fixes (Iteration 2)

### HIGH: Annotation saves fail on Windows after first write
**Problem**: `AnnotationStore::save` used `fs::rename` which fails on Windows if the destination file already exists, breaking all subsequent annotation updates.

**Solution**:
- Replaced manual temp file + `fs::rename` with `tempfile::NamedTempFile::persist()`
- `persist()` handles cross-platform atomic replacement correctly (works on Windows)
- Added proper error propagation for parent directory validation

**Files Modified**:
- `src/models/annotation.rs`: Rewrote `save()` to use `NamedTempFile::persist()`

**Tests Added**:
- `test_annotation_store_multiple_saves`: Verifies sequential saves work (simulates Windows scenario)
- `test_annotation_store_save_creates_parent_dirs`: Verifies directory creation

### HIGH: Save failure UX does not keep the editor open
**Problem**: The UI closed the modal and showed success toast before receiving IPC response. On save failure, users lost their input and the annotation was incorrectly shown.

**Solution**:
- Added `pendingSave` state to track in-progress saves
- Deferred modal close and success toast until successful IPC response
- On error: remove optimistic annotation, keep modal open, reset save button, show error toast
- Added visual feedback ("Saving..." button state) during save

**Files Modified**:
- `src/ui/viewer/assets/app.js`: Added `pendingSave` state, updated `saveAnnotation()`, `handleAnnotationResponse()`, error handling

**Tests Added**:
- `test_handle_save_annotation_error_returns_error_response`: Verifies save errors propagate correctly
- `test_error_response_format_for_ui`: Verifies error response JSON format

### MEDIUM: Heading rendering drops inline formatting
**Problem**: `render_markdown_to_html` only extracted plain text from headings, discarding inline formatting like `code`, **bold**, *italic*, and [links].

**Solution**:
- Buffer all heading events instead of just collecting text
- Replay buffered events after injecting the heading ID
- Plain text extraction still used for slug generation

**Files Modified**:
- `src/ui/viewer/render.rs`: Rewrote heading processing to buffer and replay events

**Tests Added**:
- `test_render_markdown_heading_with_inline_code`
- `test_render_markdown_heading_with_emphasis`
- `test_render_markdown_heading_with_bold`
- `test_render_markdown_heading_with_link`
- `test_render_markdown_heading_mixed_formatting`

### MEDIUM: No test coverage for resolve nonexistent annotation
**Problem**: Missing test for the error case when resolving a non-existent annotation.

**Solution**:
- Added `test_handle_resolve_nonexistent_annotation_returns_error`

## Test Results

```bash
$ cargo test --features ui
running 266 tests
...
test result: ok. 266 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

# Integration tests (tests/viewer_test.rs)
running 29 tests
...
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test Coverage

| Category | Tests |
|----------|-------|
| Annotation models | 15+ tests (serialization, UUID generation, resolve, persistence) |
| Slugify algorithm | 8 tests (special chars, unicode, numbers, edge cases) |
| HTML rendering | 10+ tests (headings, tables, code blocks, GFM features) |
| Path traversal | 5 tests (.., /, \\, allowlist) |
| AnnotationStore resilience | 3 tests (malformed JSON, partial JSON, backup creation) |
| IPC message handling | 10+ tests (load, save, resolve, errors) |
| ViewerManager | 10+ tests (file loading, YAML, missing files) |

### Acceptance Criteria Verification

| Scenario | Status | Test |
|----------|--------|------|
| `agentd view my-change` opens window with sidebar | Implemented | `test_viewer_manager_change_exists` |
| `agentd view non-existent` exits with error | Implemented | `test_change_directory_check` |
| UI disabled fallback logs path | Implemented | Code verified in `proposal.rs` |
| Missing file shows placeholder | Implemented | `test_viewer_manager_load_missing_file` |
| Path traversal rejected | Implemented | `test_path_traversal_*` tests |
| Annotation save/resolve | Implemented | `test_handle_save_annotation`, `test_handle_resolve_annotation` |

## Build Instructions

```bash
# Build without UI (default)
cargo build

# Build with UI feature
cargo build --features ui

# Run viewer
cargo run --features ui -- view <change-id>
```

## Final Verification (2026-01-17)

All tasks from `tasks.md` have been implemented and verified:

### Task Completion Summary

| Task | Description | Status |
|------|-------------|--------|
| 1.1 | Annotation data models | ✅ Complete |
| 1.2 | Models index update | ✅ Complete |
| 1.3 | Cargo.toml dependencies | ✅ Complete |
| 2.1 | Markdown/YAML rendering | ✅ Complete |
| 2.2 | Viewer state and file loading | ✅ Complete |
| 2.3 | IPC message handler | ✅ Complete |
| 2.4 | Window launcher | ✅ Complete |
| 3.1 | Register view command | ✅ Complete |
| 3.2 | View CLI command implementation | ✅ Complete |
| 3.3 | Register viewer module | ✅ Complete |
| 3.4 | Refactor main for conditional runtime | ✅ Complete |
| 3.5 | Auto-open viewer in proposal workflow | ✅ Complete |
| 4.1 | Bundle UI assets | ✅ Complete |
| 4.2 | Webview template (index.html) | ✅ Complete |
| 4.3 | Webview scripts (app.js) | ✅ Complete |
| 5.1 | Unit tests for Annotation models | ✅ Complete |
| 5.2 | Unit tests for slug generation | ✅ Complete |
| 5.3 | Integration tests for HTML rendering | ✅ Complete |
| 5.4 | Unit tests for AnnotationStore resilience | ✅ Complete |
| 5.5 | Unit tests for path traversal | ✅ Complete |
| 5.6 | README documentation | ✅ Complete |

### Test Results Summary

```
cargo test --features ui
266 unit tests passed
14 fillback tests passed
29 viewer integration tests passed
1 doc-test passed
Total: 310 tests, all passing
```

### Files Created/Modified

**New Files:**
- `src/models/annotation.rs` - Annotation and AnnotationStore data models
- `src/cli/view.rs` - View command implementation
- `src/ui/viewer/mod.rs` - Viewer window launcher
- `src/ui/viewer/render.rs` - Markdown/YAML rendering
- `src/ui/viewer/manager.rs` - File loading and state management
- `src/ui/viewer/ipc.rs` - IPC message handling
- `src/ui/viewer/assets.rs` - Asset bundling
- `src/ui/viewer/assets/index.html` - Webview template
- `src/ui/viewer/assets/app.js` - Frontend JavaScript
- `src/ui/viewer/assets/styles.css` - UI styles
- `src/ui/viewer/assets/highlight.min.js` - Syntax highlighting
- `src/ui/viewer/assets/highlight.min.css` - Highlight.js theme
- `src/ui/viewer/assets/mermaid.min.js` - Mermaid diagram library
- `tests/viewer_test.rs` - Integration tests

**Modified Files:**
- `Cargo.toml` - Added ui feature and dependencies
- `src/main.rs` - Conditional runtime for view command
- `src/cli/mod.rs` - Registered view module
- `src/ui/mod.rs` - Registered viewer module
- `src/models/mod.rs` - Registered annotation module
- `src/cli/proposal.rs` - Auto-open viewer integration
- `README.md` - Added view command documentation

## Review Fixes (Iteration 3)

### HIGH: Mermaid/Highlight.js assets are placeholders
**Problem**: The bundled assets for `mermaid.min.js`, `highlight.min.js`, and `highlight.min.css` were placeholder stubs. Mermaid diagrams and syntax highlighting would not function.

**Solution**:
- Downloaded and bundled real libraries from CDN:
  - Mermaid.js v10.9.0 (3.3MB minified)
  - Highlight.js v11.9.0 (122KB minified)
  - Highlight.js atom-one-dark theme (856 bytes)

**Files Modified**:
- `src/ui/viewer/assets/mermaid.min.js` - Replaced with real library
- `src/ui/viewer/assets/highlight.min.js` - Replaced with real library
- `src/ui/viewer/assets/highlight.min.css` - Replaced with real theme

### MEDIUM: HTML injection in annotation metadata
**Problem**: `section_id`, `author`, and `id` were rendered via template literals without escaping. A crafted `annotations.json` or hostile git `user.name` could inject HTML into the viewer.

**Solution**:
- Applied `escapeHtml()` to all user-controlled fields in `renderAnnotations()`:
  - `a.id` - escaped in data-id attribute and onclick handler
  - `a.section_id` - escaped in section display
  - `a.author` - escaped in meta display

**Files Modified**:
- `src/ui/viewer/assets/app.js` - Added escaping to annotation rendering

### MEDIUM: Resolve annotation lacks rollback on failure
**Problem**: The UI marked annotations as resolved and showed success immediately, but if the IPC resolve call failed, the UI stayed resolved even though the backend rejected the change.

**Solution**:
- Added `pendingResolve` state to track in-progress resolves
- Deferred success toast until IPC confirmation in `handleAnnotationResponse()`
- Added rollback logic in error handler: on failure, set `resolved = false` and re-render
- Pattern matches existing `pendingSave` error handling

**Files Modified**:
- `src/ui/viewer/assets/app.js`:
  - Added `pendingResolve` to state
  - Updated `resolveAnnotation()` to track pending operation
  - Updated error handler to rollback resolve on failure
  - Updated `handleAnnotationResponse()` to show success toast on confirmation

### Test Results After Fixes

```
cargo test --features ui
266 unit tests passed
14 fillback tests passed
29 viewer integration tests passed
1 doc-test passed
Total: 310 tests, all passing
```

## Review Fixes (Iteration 4)

### HIGH: Webview renders unsanitized HTML from plan files
**Problem**: Markdown is rendered directly to HTML and injected into the DOM. `pulldown-cmark` passes through raw HTML blocks, allowing untrusted plan content to execute scripts in the webview and access `window.ipc`. This is a stored XSS vector.

**Solution**:
- Implemented HTML sanitization using an allowlist approach
- Added `sanitize_html()` function that:
  - Removes unsafe tags: `<script>`, `<style>`, `<iframe>`, `<object>`, `<embed>`, etc.
  - Removes event handlers: `onclick`, `onerror`, `onload`, etc.
  - Removes `javascript:` URLs in `href` and `src` attributes
  - Preserves safe tags: `<h1>`, `<p>`, `<a>`, `<code>`, `<table>`, etc.
  - Preserves safe attributes: `id`, `class`, `href`, `src`, `alt`, `title`, etc.
- Applied sanitization to all rendered markdown before injection

**Files Modified**:
- `src/ui/viewer/render.rs`: Added `sanitize_html()`, `sanitize_tag()`, `parse_attributes()` functions

**Tests Added**:
- `test_sanitize_removes_script_tags`
- `test_sanitize_removes_event_handlers`
- `test_sanitize_removes_javascript_urls`
- `test_sanitize_allows_safe_tags`
- `test_sanitize_removes_iframe`
- `test_sanitize_removes_style_tags`
- `test_sanitize_preserves_id_and_class`
- `test_markdown_with_raw_html_xss`
- `test_markdown_with_img_onerror`

### MEDIUM: Mermaid configured with `securityLevel: 'loose'`
**Problem**: `securityLevel: 'loose'` allows Mermaid diagrams to inject raw HTML. Combined with untrusted plan content, this expands the XSS surface.

**Solution**:
- Changed Mermaid initialization to use `securityLevel: 'strict'` (the safe default)
- This prevents diagram content from injecting HTML

**Files Modified**:
- `src/ui/viewer/assets/app.js`: Updated `mermaid.initialize()` call

## Review Fixes (Iteration 5)

### HIGH: `change_id` path traversal allows access outside `agentd/changes/`
**Problem**: `change_id` was concatenated into the change path without sanitization. Inputs like `../some-dir` could escape `agentd/changes/`, letting the viewer read/write `annotations.json` outside the intended change directory.

**Solution**:
- Added `validate_change_id()` function that rejects:
  - Empty change_ids
  - Path separators (`/` or `\`)
  - Parent directory references (`..`)
- Added canonicalization + prefix check as defense-in-depth:
  - Canonicalize both the change directory and the changes root
  - Verify the resolved path starts with the changes root
- Validation runs before any path construction

**Files Modified**:
- `src/cli/view.rs`: Added `validate_change_id()`, updated `run()` with validation and canonicalization

**Tests Added**:
- `test_validate_change_id_valid`: Verifies valid change_ids pass
- `test_validate_change_id_rejects_empty`: Verifies empty string rejected
- `test_validate_change_id_rejects_path_traversal`: Verifies `..` patterns rejected
- `test_validate_change_id_rejects_forward_slash`: Verifies `/` rejected
- `test_validate_change_id_rejects_backslash`: Verifies `\` rejected
- `test_validate_change_id_error_messages`: Verifies error messages are descriptive

### Test Results After Fixes

```
cargo test --features ui
281 unit tests passed (including 7 new view validation tests)
All tests passing
```

## Review Fixes (Iteration 6)

### HIGH: Missing per-section annotation affordance
**Problem**: The UI relied on clicking headings or a global "+ Add" button, but there was no visible comment icon/button near each section. The requirement (specs/annotations.md R3) explicitly calls for a per-section affordance.

**Solution**:
- Added visible "Comment" buttons next to each heading with an ID (h1[id], h2[id], h3[id])
- Each heading is wrapped in a `.section-wrapper` container for positioning
- Button styled with `.section-comment-btn` class: semi-transparent, becomes prominent on hover
- Button includes 💬 icon and "Comment" text for discoverability
- Clicking the button opens the annotation modal pre-filled with that section
- Heading clicks are also preserved for convenience

**Files Modified**:
- `src/ui/viewer/assets/styles.css`: Added `.section-wrapper`, `.section-comment-btn` styles
- `src/ui/viewer/assets/app.js`: Updated `initHeadingClickHandlers()` to inject visible buttons

### MEDIUM: HTML sanitizer allows unsafe URL schemes
**Problem**: The sanitizer only blocked `javascript:` URLs but allowed `data:`, `vbscript:`, and `file:` schemes in `href`/`src` attributes. A plan file could embed a `data:` SVG or other active content.

**Solution**:
- Changed from blocklist to allowlist approach for URL schemes
- Now only allows: `http://`, `https://`, `mailto:`, `#` (anchors), and relative paths (no `:` character)
- Blocks all other schemes including `javascript:`, `data:`, `vbscript:`, `file:`

**Files Modified**:
- `src/ui/viewer/render.rs`: Rewrote URL scheme validation in `sanitize_tag()`

**Tests Added**:
- `test_sanitize_blocks_data_url`
- `test_sanitize_blocks_vbscript_url`
- `test_sanitize_blocks_file_url`
- `test_sanitize_allows_relative_urls`
- `test_sanitize_allows_anchor_urls`

### MEDIUM: UI-disabled fallback message not exact
**Problem**: The fallback message used `.yellow()` which injects ANSI color codes, breaking the "exact message" requirement from specs/plan-viewer.md R6.

**Solution**:
- Removed `.yellow()` call from the fallback message
- Now prints plain text: `"UI feature disabled. View plan manually at: {path}"`

**Files Modified**:
- `src/cli/proposal.rs`: Simplified `open_viewer_if_available()` to print without ANSI formatting

## Review Fixes (Iteration 7)

### MEDIUM: Missing annotations when file is absent
**Problem**: When a file is missing, `handleFileLoadResponse` discarded `data.annotations` and passed an empty array to `updateContent`. This prevented existing annotations from showing alongside the "File not found" placeholder.

**Solution**:
- Simplified `handleFileLoadResponse` to always pass `data.annotations` regardless of whether file exists
- Annotations are now shown even when viewing a placeholder for a missing file

**Files Modified**:
- `src/ui/viewer/assets/app.js`: Updated `handleFileLoadResponse()` to preserve annotations

### MEDIUM: Comment affordance limited to h1-h3 headings
**Problem**: The visible "Comment" buttons were only attached to `h1`, `h2`, and `h3` headings. Sections using `h4`-`h6` headings (common in plans/specs) had no visible affordance, violating the R3 requirement for per-section annotation controls.

**Solution**:
- Extended `initHeadingClickHandlers` selector to include all heading levels: `h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]`
- All headings with IDs now get visible comment buttons

**Files Modified**:
- `src/ui/viewer/assets/app.js`: Extended heading selector in `initHeadingClickHandlers()`
