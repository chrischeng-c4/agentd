---
id: merge-viewer-to-mcp
type: tasks
version: 1
---

# Tasks

## 1. Data Layer

- [ ] 1.1 Define Dashboard and Config Injection Structures
  - File: `src/mcp/http_server.rs` (MODIFY)
  - Spec: `specs/server-unification.md#data-model`
  - Do: Define `DashboardState` and `InjectedConfig` structs for JSON serialization. `DashboardState` should include project lists and server info. `InjectedConfig` should contain `base_path`, `project`, and `change_id`.
  - Depends: none

## 2. Logic Layer

- [ ] 2.1 Refactor Viewer Router and Lifecycle
  - File: `src/ui/viewer/mod.rs` (MODIFY)
  - Spec: `specs/server-unification.md#interfaces`
  - Do: Remove `start_viewer`, `run_server`, and `AppState`. Create a function `create_viewer_router(registry: Arc<Registry>) -> Router`. Ensure handlers use the shared registry to resolve project paths dynamically.
  - Depends: 1.1

- [ ] 2.2 Implement Scoped API Handlers with Validation
  - File: `src/ui/viewer/mod.rs` (MODIFY)
  - Spec: `specs/server-unification.md#r8-registry-based-data-resolution`
  - Do: Update all API handlers (info, files, annotations, etc.) to accept `:project` and `:change` as path parameters. Use the `Registry` to validate that the project exists and is registered. Implement 404 responses for invalid projects/changes.
  - Depends: 2.1

- [ ] 2.3 Implement Configuration Injection and Asset Routing
  - File: `src/ui/viewer/mod.rs` (MODIFY)
  - Spec: `specs/server-unification.md#r5-configuration-injection`
  - Do: Update `serve_index` handler to perform string replacement on `index.html` to inject the `<script id="agentd-config">` tag. Create a `/static` route using `Router::nest` or similar to serve all embedded assets (CSS, JS).
  - Depends: 2.2

- [ ] 2.4 Implement Dashboard Handler Logic
  - File: `src/mcp/http_server.rs` (MODIFY)
  - Spec: `specs/server-unification.md#r6-server-dashboard`
  - Do: Implement the `handle_dashboard` function. It should iterate through the `Registry`, scan the `agentd/changes/` directory for each project to find active changes, and return the `DashboardState`.
  - Depends: 1.1

## 3. Integration

- [ ] 3.1 Combine MCP and Viewer Routes
  - File: `src/mcp/http_server.rs` (MODIFY)
  - Spec: `specs/server-unification.md#r2-combined-http-server`
  - Do: Update `start_server` to mount the Dashboard at `/`, the MCP endpoint at `/mcp`, and the Viewer router at `/view/:project/:change`.
  - Depends: 2.1, 2.4

- [ ] 3.2 Rename and Move MCP Management to Server CLI
  - File: `src/cli/mcp_server_mgmt.rs` (DELETE)
  - File: `src/cli/server.rs` (CREATE)
  - Spec: `specs/server-unification.md#r1-unified-subcommand`
  - Do: Move functionality from `mcp_server_mgmt.rs` to `server.rs`. Rename `McpServerCommands` to `ServerCommands` and update references. Update command help text to reflect the unified nature.
  - Depends: none

- [ ] 3.3 Update Main CLI and Remove Standalone View
  - File: `src/main.rs` (MODIFY)
  - File: `src/cli/view.rs` (DELETE)
  - File: `src/cli/mod.rs` (MODIFY)
  - Spec: `specs/server-unification.md#r1-unified-subcommand`
  - Do: Rename `McpServer` subcommand to `Server`. Remove the top-level `View` command and its special main-thread handling. Clean up module registration in `mod.rs`.
  - Depends: 3.2

- [ ] 3.4 Implement Server View Subcommand and Output Updates
  - File: `src/cli/server.rs` (MODIFY)
  - Spec: `specs/server-unification.md#r7-cli-convenience-command`
  - Do: Add `view <project> <change>` command that opens the scoped URL in the browser. Update `server start` success message to display the dashboard URL.
  - Depends: 3.2

- [ ] 3.5 Update Frontend Assets and Scoped API Calls
  - File: `src/ui/viewer/assets/app.js` (MODIFY)
  - File: `src/ui/viewer/assets/index.html` (MODIFY)
  - Spec: `specs/server-unification.md#r5-configuration-injection`
  - Do: Modify `app.js` to read the `base_path` from the injected config and prepend it to all API `fetch` calls. Update `index.html` to use absolute `/static/` paths for all scripts and styles.
  - Depends: 2.3

## 4. Testing

- [ ] 4.1 Verify Unified Server Routing and Isolation
  - File: `tests/server_unification_test.rs` (CREATE)
  - Verify: `specs/server-unification.md#acceptance-criteria`
  - Do: Create integration tests that start the server and verify that MCP, Dashboard, and Scoped Viewer (including 404 cases) all work correctly on the same port.
  - Depends: 3.1
