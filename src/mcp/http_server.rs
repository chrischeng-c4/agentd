//! Unified HTTP Server - MCP + Dashboard + Plan Viewer
//!
//! This server combines:
//! - MCP JSON-RPC endpoint at `/mcp`
//! - Dashboard at `/`
//! - Scoped Plan Viewer at `/view/:project/:change` (requires `ui` feature)
//! - Static assets at `/static/*` (requires `ui` feature)
//!
//! ## Features
//! - Project isolation via URL scoping and X-Agentd-Project header
//! - Multi-project support with single server instance
//! - Configuration injection for frontend routing

use crate::mcp::server::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::mcp::{McpServer, Registry};
use crate::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[cfg(feature = "ui")]
use axum::{extract::Path, http::header};
#[cfg(feature = "ui")]
use serde::Deserialize;

// =============================================================================
// Data Models (R6 Dashboard, R5 Config Injection)
// =============================================================================

/// Dashboard state showing all registered projects and their changes
#[derive(Debug, Clone, Serialize)]
pub struct DashboardState {
    pub server_info: ServerInfo,
    pub projects: Vec<ProjectInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub port: u16,
    pub pid: u32,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub changes: Vec<ChangeInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeInfo {
    pub id: String,
    pub status: String,
}

/// Configuration injected into viewer HTML (R5)
#[cfg(feature = "ui")]
#[derive(Debug, Clone, Serialize)]
pub struct InjectedConfig {
    pub base_path: String,
    pub project: String,
    pub change_id: String,
}

// =============================================================================
// Application State
// =============================================================================

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Project registry (thread-safe)
    pub registry: Arc<RwLock<Registry>>,
}

impl AppState {
    pub fn new(registry: Registry) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }
}

// JSON-RPC error codes
const INVALID_REQUEST: i32 = -32600;
const INTERNAL_ERROR: i32 = -32603;

// =============================================================================
// Server Startup (R2 Combined HTTP Server)
// =============================================================================

/// Start unified HTTP server with MCP, Dashboard, and Viewer routes
pub async fn start_server(port: u16, registry: Registry) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let state = AppState::new(registry);

    // Build base router with MCP and dashboard
    #[allow(unused_mut)]
    let mut app = Router::new()
        // Dashboard at root (R6)
        .route("/", get(handle_dashboard))
        .route("/api/dashboard", get(api_dashboard))
        // MCP endpoint (existing)
        .route("/mcp", post(handle_mcp_request))
        // Health check
        .route("/health", get(health_check));

    // Add viewer routes if ui feature is enabled
    #[cfg(feature = "ui")]
    {
        app = app
            // Static assets (R4)
            .route("/static/styles.css", get(serve_styles))
            .route("/static/app.js", get(serve_app_js))
            .route("/static/highlight.min.css", get(serve_highlight_css))
            .route("/static/highlight.min.js", get(serve_highlight_js))
            .route("/static/mermaid.min.js", get(serve_mermaid_js))
            // Scoped viewer routes (R3)
            .route("/view/:project/:change", get(serve_viewer_html))
            .route("/view/:project/:change/", get(serve_viewer_html))
            .route("/view/:project/:change/api/info", get(api_viewer_info))
            .route("/view/:project/:change/api/files", get(api_viewer_files))
            .route(
                "/view/:project/:change/api/files/*path",
                get(api_viewer_load_file),
            )
            .route(
                "/view/:project/:change/api/annotations",
                post(api_viewer_save_annotation),
            )
            .route(
                "/view/:project/:change/api/annotations/:id/resolve",
                post(api_viewer_resolve_annotation),
            )
            .route(
                "/view/:project/:change/api/review/approve",
                post(api_viewer_approve),
            )
            .route(
                "/view/:project/:change/api/review/request-changes",
                post(api_viewer_request_changes),
            )
            .route("/view/:project/:change/api/close", post(api_viewer_close));
    }

    let app = app.layer(CorsLayer::permissive()).with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("✓ Server listening on http://{}", addr);
    println!("  Dashboard: http://{}/", addr);
    println!("  MCP endpoint: http://{}/mcp", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

// =============================================================================
// Dashboard Handlers (R6)
// =============================================================================

/// Serve dashboard HTML page
async fn handle_dashboard() -> Html<&'static str> {
    Html(include_str!("dashboard.html"))
}

/// Dashboard API - returns project and change listing
async fn api_dashboard(State(state): State<AppState>) -> Response {
    // Reload registry to get latest state
    let registry = match Registry::load() {
        Ok(reg) => reg,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to load registry: {}", e) })),
            )
                .into_response();
        }
    };

    // Update shared state
    {
        let mut reg_lock = state.registry.write().await;
        *reg_lock = registry.clone();
    }

    // Build dashboard state
    let mut projects = Vec::new();
    for (name, info) in registry.list_projects() {
        let changes = scan_project_changes(&info.path);
        projects.push(ProjectInfo {
            name: name.clone(),
            path: info.path.display().to_string(),
            changes,
        });
    }

    let dashboard = DashboardState {
        server_info: ServerInfo {
            port: registry.server.port,
            pid: registry.server.pid,
            started_at: registry.server.started_at.to_rfc3339(),
        },
        projects,
    };

    Json(dashboard).into_response()
}

/// Scan a project directory for active changes
fn scan_project_changes(project_path: &std::path::Path) -> Vec<ChangeInfo> {
    let changes_dir = project_path.join("agentd/changes");
    let mut changes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&changes_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let id = entry.file_name().to_string_lossy().to_string();
                let status = read_change_status(&entry.path());
                changes.push(ChangeInfo { id, status });
            }
        }
    }

    changes
}

/// Read status from STATE.yaml
fn read_change_status(change_dir: &std::path::Path) -> String {
    let state_file = change_dir.join("STATE.yaml");
    if state_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&state_file) {
            // Simple YAML parsing for phase field
            for line in content.lines() {
                if line.starts_with("phase:") {
                    return line.trim_start_matches("phase:").trim().to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

// =============================================================================
// Static Asset Handlers (R4) - requires ui feature
// =============================================================================

#[cfg(feature = "ui")]
async fn serve_styles() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../ui/viewer/assets/styles.css"),
    )
}

#[cfg(feature = "ui")]
async fn serve_app_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        include_str!("../ui/viewer/assets/app.js"),
    )
}

#[cfg(feature = "ui")]
async fn serve_highlight_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("../ui/viewer/assets/highlight.min.css"),
    )
}

#[cfg(feature = "ui")]
async fn serve_highlight_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        include_str!("../ui/viewer/assets/highlight.min.js"),
    )
}

#[cfg(feature = "ui")]
async fn serve_mermaid_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        include_str!("../ui/viewer/assets/mermaid.min.js"),
    )
}

// =============================================================================
// Scoped Viewer Handlers (R3, R5, R8) - requires ui feature
// =============================================================================

/// Path parameters for scoped viewer routes
#[cfg(feature = "ui")]
#[derive(Deserialize)]
struct ViewerPath {
    project: String,
    change: String,
}

/// Serve viewer HTML with injected configuration (R5)
#[cfg(feature = "ui")]
async fn serve_viewer_html(
    State(state): State<AppState>,
    Path(params): Path<ViewerPath>,
) -> Response {
    // Validate project and change (R8)
    match validate_project_change(&state, &params.project, &params.change).await {
        Ok(_) => {}
        Err(response) => return response,
    }

    // Create injected config (R5)
    let config = InjectedConfig {
        base_path: format!("/view/{}/{}/api", params.project, params.change),
        project: params.project.clone(),
        change_id: params.change.clone(),
    };
    let config_json = serde_json::to_string(&config).unwrap_or_default();

    // Inject config into HTML
    let html = include_str!("../ui/viewer/assets/index.html");
    let injected_html = html.replace(
        "</head>",
        &format!(
            r#"<script id="agentd-config" type="application/json">{}</script></head>"#,
            config_json
        ),
    );

    Html(injected_html).into_response()
}

/// Validate project exists in registry and change exists on filesystem (R8)
#[cfg(feature = "ui")]
async fn validate_project_change(
    state: &AppState,
    project: &str,
    change: &str,
) -> std::result::Result<PathBuf, Response> {
    // Reload registry
    let registry = Registry::load().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to load registry: {}", e) })),
        )
            .into_response()
    })?;

    // Update shared state
    {
        let mut reg_lock = state.registry.write().await;
        *reg_lock = registry.clone();
    }

    // Check project exists in registry
    let project_path = registry.get_project_path(project).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Project '{}' not registered", project)
            })),
        )
            .into_response()
    })?;

    // Check change exists on filesystem
    let change_dir = project_path.join("agentd/changes").join(change);
    if !change_dir.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Change '{}' not found in project '{}'", change, project)
            })),
        )
            .into_response());
    }

    Ok(change_dir)
}

/// Get ViewerManager for a validated project/change
#[cfg(feature = "ui")]
fn get_viewer_manager(
    project_path: &std::path::Path,
    change: &str,
) -> crate::ui::viewer::ViewerManager {
    crate::ui::viewer::ViewerManager::new(change, project_path)
}

/// API: Get viewer info
#[cfg(feature = "ui")]
async fn api_viewer_info(
    State(state): State<AppState>,
    Path(params): Path<ViewerPath>,
) -> Response {
    let change_dir = match validate_project_change(&state, &params.project, &params.change).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    // Get project path from change_dir (two levels up from change dir)
    let project_path = change_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&change_dir);
    let manager = get_viewer_manager(project_path, &params.change);

    #[derive(Serialize)]
    struct InfoResponse {
        change_id: String,
        project: String,
        files: Vec<crate::ui::viewer::FileInfo>,
    }

    Json(InfoResponse {
        change_id: params.change.clone(),
        project: params.project.clone(),
        files: manager.list_files(),
    })
    .into_response()
}

/// API: List files
#[cfg(feature = "ui")]
async fn api_viewer_files(
    State(state): State<AppState>,
    Path(params): Path<ViewerPath>,
) -> Response {
    let change_dir = match validate_project_change(&state, &params.project, &params.change).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let project_path = change_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&change_dir);
    let manager = get_viewer_manager(project_path, &params.change);

    Json(manager.list_files()).into_response()
}

/// Path for file loading (includes wildcard path)
#[cfg(feature = "ui")]
#[derive(Deserialize)]
struct ViewerFilePath {
    project: String,
    change: String,
    path: String,
}

/// API: Load file content
#[cfg(feature = "ui")]
async fn api_viewer_load_file(
    State(state): State<AppState>,
    Path(params): Path<ViewerFilePath>,
) -> Response {
    let change_dir = match validate_project_change(&state, &params.project, &params.change).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let project_path = change_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&change_dir);
    let manager = get_viewer_manager(project_path, &params.change);

    let filename = params.path.trim_start_matches('/');
    match manager.load_file(filename) {
        Ok(response) => Json(response).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Annotation save request
#[cfg(feature = "ui")]
#[derive(Deserialize)]
struct SaveAnnotationRequest {
    file: String,
    section_id: String,
    content: String,
}

/// API: Save annotation
#[cfg(feature = "ui")]
async fn api_viewer_save_annotation(
    State(state): State<AppState>,
    Path(params): Path<ViewerPath>,
    Json(req): Json<SaveAnnotationRequest>,
) -> Response {
    use crate::models::{get_author_name, Annotation};

    let change_dir = match validate_project_change(&state, &params.project, &params.change).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let project_path = change_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&change_dir);
    let manager = get_viewer_manager(project_path, &params.change);

    let author = get_author_name();
    let annotation = Annotation::new(&req.file, &req.section_id, &req.content, author);

    match manager.load_annotations() {
        Ok(mut store) => {
            let annotation_clone = annotation.clone();
            store.add(annotation);
            match manager.save_annotations(&store) {
                Ok(_) => Json(annotation_clone).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Annotation path with ID
#[cfg(feature = "ui")]
#[derive(Deserialize)]
struct AnnotationPath {
    project: String,
    change: String,
    id: String,
}

/// API: Resolve annotation
#[cfg(feature = "ui")]
async fn api_viewer_resolve_annotation(
    State(state): State<AppState>,
    Path(params): Path<AnnotationPath>,
) -> Response {
    let change_dir = match validate_project_change(&state, &params.project, &params.change).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let project_path = change_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&change_dir);
    let manager = get_viewer_manager(project_path, &params.change);

    match manager.load_annotations() {
        Ok(mut store) => {
            if let Err(e) = store.resolve(&params.id) {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response();
            }

            match manager.save_annotations(&store) {
                Ok(_) => {
                    let annotation = store.find(&params.id).cloned();
                    Json(annotation).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// API: Approve review
#[cfg(feature = "ui")]
async fn api_viewer_approve(
    State(state): State<AppState>,
    Path(params): Path<ViewerPath>,
) -> Response {
    let change_dir = match validate_project_change(&state, &params.project, &params.change).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let project_path = change_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&change_dir);
    let manager = get_viewer_manager(project_path, &params.change);

    match manager.update_phase("complete") {
        Ok(_) => Json(serde_json::json!({
            "action": "approve_review",
            "status": "success",
            "message": "Review approved. Phase updated to complete."
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "action": "approve_review",
                "status": "error",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// API: Request changes
#[cfg(feature = "ui")]
async fn api_viewer_request_changes(
    State(state): State<AppState>,
    Path(params): Path<ViewerPath>,
) -> Response {
    let change_dir = match validate_project_change(&state, &params.project, &params.change).await {
        Ok(dir) => dir,
        Err(response) => return response,
    };

    let project_path = change_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&change_dir);
    let manager = get_viewer_manager(project_path, &params.change);

    if let Err(e) = manager.update_phase("changes_requested") {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "action": "request_changes",
                "status": "error",
                "message": e.to_string()
            })),
        )
            .into_response();
    }

    match manager.load_annotations() {
        Ok(store) => {
            let unresolved_count = store.unresolved_count();
            Json(serde_json::json!({
                "action": "request_changes",
                "status": "success",
                "message": format!("Changes requested with {} comment(s). Phase updated.", unresolved_count)
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "action": "request_changes",
                "status": "error",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// API: Close viewer (no-op for unified server, just returns success)
#[cfg(feature = "ui")]
async fn api_viewer_close() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "action": "close_window",
        "status": "success"
    }))
}

// =============================================================================
// MCP Handler (existing)
// =============================================================================

/// Main MCP request handler
async fn handle_mcp_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    // Extract project from header
    let project_name = match extract_project_name(&headers) {
        Ok(name) => name,
        Err(e) => {
            return error_response(
                request.id,
                INVALID_REQUEST,
                format!("Missing or invalid X-Agentd-Project header: {}", e),
            );
        }
    };

    // Reload registry from disk to pick up new registrations
    let fresh_registry = match Registry::load() {
        Ok(reg) => reg,
        Err(e) => {
            return error_response(
                request.id,
                INTERNAL_ERROR,
                format!("Failed to load registry: {}", e),
            );
        }
    };

    // Update shared state with fresh registry
    {
        let mut registry_lock = state.registry.write().await;
        *registry_lock = fresh_registry.clone();
    }

    // Get project path from fresh registry
    let project_path = match fresh_registry.get_project_path(&project_name) {
        Some(path) => path.clone(),
        None => {
            return error_response(
                request.id,
                INVALID_REQUEST,
                format!("Project '{}' not registered", project_name),
            );
        }
    };

    // Execute request in project context
    match execute_in_project_context(request, project_path).await {
        Ok(Some(response)) => Json(response).into_response(),
        Ok(None) => {
            // Notification - return HTTP 202 Accepted with empty body
            (axum::http::StatusCode::ACCEPTED, "").into_response()
        }
        Err(e) => {
            eprintln!("Error executing request: {}", e);
            error_response(
                Some(Value::Null),
                INTERNAL_ERROR,
                format!("Internal error: {}", e),
            )
        }
    }
}

/// Extract project name from headers
fn extract_project_name(headers: &HeaderMap) -> Result<String> {
    let header_value = headers
        .get("X-Agentd-Project")
        .or_else(|| headers.get("x-agentd-project"))
        .ok_or_else(|| anyhow::anyhow!("X-Agentd-Project header not found"))?;

    let project_name = header_value
        .to_str()
        .map_err(|_| anyhow::anyhow!("Invalid header value"))?
        .to_string();

    Ok(project_name)
}

/// Execute MCP request in project context
async fn execute_in_project_context(
    request: JsonRpcRequest,
    project_path: PathBuf,
) -> Result<Option<JsonRpcResponse>> {
    // Change to project directory
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&project_path)?;

    // Create MCP server instance for this project
    let server = McpServer::new()?;

    // Handle the request
    let response = server.handle_request_json(&request).await;

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    Ok(response)
}

/// Create error response
fn error_response(id: Option<Value>, code: i32, message: String) -> Response {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: id.unwrap_or(Value::Null),
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_project_name() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Agentd-Project", "test-project".parse().unwrap());

        let name = extract_project_name(&headers).unwrap();
        assert_eq!(name, "test-project");
    }

    #[test]
    fn test_extract_project_name_missing() {
        let headers = HeaderMap::new();
        let result = extract_project_name(&headers);
        assert!(result.is_err());
    }

    #[cfg(feature = "ui")]
    #[test]
    fn test_injected_config_serialization() {
        let config = InjectedConfig {
            base_path: "/view/myproj/change-1/api".to_string(),
            project: "myproj".to_string(),
            change_id: "change-1".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("base_path"));
        assert!(json.contains("myproj"));
    }
}
