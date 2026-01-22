# Change: merge-viewer-to-mcp

## Summary

Consolidate the standalone `plan viewer` and `mcp-server` into a single, unified `agentd server` command. This change removes the independent `view` command and provides both MCP tools and a web-based plan viewer through a single HTTP server.

## Why

1.  **Unified Infrastructure**: Both the MCP server and Plan Viewer currently use separate `axum` server implementations. Merging them reduces code duplication and maintenance overhead.
2.  **Simplified CLI**: Reducing the number of top-level commands makes the tool easier to learn and use. `agentd server` becomes the central hub for all remote/web interactions.
3.  **Better User Experience**: A single long-running server can manage multiple projects and changes. Users can switch between reviewing different plans without restarting the viewer or changing ports.
4.  **Agent-User Synergy**: Future features could allow agents to "push" a view to the user through the same connection.

## What Changes

- **CLI Interface**:
    - Rename `agentd mcp-server` subcommand to `agentd server`.
    - Replace the standalone `agentd view <change_id>` command with `agentd server view <project> <change_id>`.
    - Update `agentd server start` to display the Web UI URL in addition to the MCP endpoint.
- **Server Core**:
    - `src/mcp/http_server.rs` will be expanded to handle web routing.
    - New routing structure:
        - `POST /mcp`: Existing MCP JSON-RPC endpoint.
        - `GET /`: Dashboard listing registered projects and active changes.
        - `GET /view/:project/:change`: Unified Plan Viewer UI.
        - `GET /view/:project/:change/api/*`: Scoped API for viewer functionality.
        - `GET /static/*`: Shared static assets (JS, CSS).
    - **Configuration Injection**: The server will inject a scoped base path into the HTML template to ensure the frontend correctly routes its API calls regardless of the project or change context.
- **Code Refactoring**:
    - Move `src/cli/mcp_server_mgmt.rs` to `src/cli/server.rs`.
    - Refactor `src/ui/viewer/mod.rs` to export an axum `Router` instead of starting its own server.
    - Remove `src/cli/view.rs`.
    - Update `src/main.rs` to reflect the new command structure.
    - Update `src/ui/viewer/assets/index.html` and `app.js` to support scoped paths.

## Impact

- Affected specs: `specs/server-unification.md` (NEW)
- Affected code:
    - `src/main.rs` (CLI definition)
    - `src/cli/mod.rs` (Module registration)
    - `src/cli/server.rs` (Moved from `mcp_server_mgmt.rs`)
    - `src/cli/view.rs` (DELETED)
    - `src/mcp/http_server.rs` (Route unification and Dashboard)
    - `src/ui/viewer/mod.rs` (Router export and injection)
    - `src/ui/viewer/assets/app.js` (API path updates)
    - `src/ui/viewer/assets/index.html` (Static path updates)
- Breaking changes: Yes. `agentd view` will no longer exist; users should use `agentd server view <project> <change>` or visit the dashboard.
