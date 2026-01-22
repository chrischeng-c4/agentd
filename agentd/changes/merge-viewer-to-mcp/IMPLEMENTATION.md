# Implementation Notes: merge-viewer-to-mcp

## Summary

Unified the Plan Viewer into the MCP server to provide a single HTTP server with:
- Dashboard at `/` for project overview
- MCP JSON-RPC at `/mcp` for AI agent communication
- Scoped Plan Viewer at `/view/:project/:change/` for per-change views
- Shared static assets at `/static/*`

## Files Modified

### New Files
- `src/mcp/http_server.rs` - Unified HTTP server with MCP, Dashboard, and Viewer routes
- `src/mcp/dashboard.html` - Dashboard HTML page
- `src/cli/server.rs` - New `agentd server` CLI commands
- `tests/server_unification_test.rs` - Integration tests

### Modified Files
- `src/cli/mod.rs` - Added `server` module
- `src/main.rs` - Added `Server` command, deprecated `McpServer`
- `src/ui/viewer/mod.rs` - Updated static asset paths to `/static/*`, re-exported `FileInfo`
- `src/ui/viewer/assets/index.html` - Updated asset paths to `/static/*`
- `src/ui/viewer/assets/app.js` - Added config injection support (R5)

## Requirements Implemented

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| R1: Replace CLI | ✅ | `agentd server` replaces `agentd mcp-server` |
| R2: Combined HTTP Server | ✅ | Single server with MCP, Dashboard, Viewer |
| R3: Scoped Viewer | ✅ | Routes at `/view/:project/:change/*` |
| R4: Shared Static Assets | ✅ | Assets served at `/static/*` |
| R5: Configuration Injection | ✅ | `InjectedConfig` with base_path |
| R6: Dashboard | ✅ | HTML + `/api/dashboard` endpoint |
| R7: server view Command | ✅ | `agentd server view <project> <change>` |
| R8: 404 Handling | ✅ | Validation in `validate_project_change` |

## Key Design Decisions

### 1. Feature Gating
Viewer-related code is gated behind `#[cfg(feature = "ui")]` to allow minimal builds without UI dependencies.

### 2. Configuration Injection
The unified server injects config via a `<script id="agentd-config">` tag:
```json
{"base_path": "/view/project/change/api", "project": "...", "change_id": "..."}
```

The frontend reads this to determine API endpoints dynamically.

### 3. Static Asset Paths
Changed from root paths (`/styles.css`) to `/static/styles.css` to avoid conflicts with scoped viewer routes.

### 4. Backwards Compatibility
- `agentd mcp-server` still works but shows deprecation warning
- Standalone `agentd view` command still works (uses same static paths)

## Testing

```bash
# Run integration tests
cargo test --test server_unification_test

# Build with UI feature
cargo build --features ui
```

## Usage

```bash
# Start unified server
agentd server start --daemon

# Open viewer for a change
agentd server view <project> <change>

# List registered projects
agentd server list

# Stop server
agentd server shutdown
```
