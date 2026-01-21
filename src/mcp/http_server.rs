//! HTTP MCP Server using Streamable HTTP transport
//!
//! This server implements the MCP protocol over HTTP, solving the stdout buffering
//! issue that occurs with stdio transport in Pipe environments.
//!
//! ## Features
//! - Project isolation via X-Agentd-Project header
//! - Multi-project support with single server instance
//! - Streamable HTTP protocol (MCP 2024-11-05+)
//! - Automatic flush (no buffering issues)

use crate::mcp::{Registry, McpServer};
use crate::mcp::server::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
use crate::Result;
use axum::{
    Router,
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::Value;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

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

/// Start HTTP MCP server
pub async fn start_server(port: u16, registry: Registry) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let state = AppState::new(registry);

    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/health", axum::routing::get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("✓ MCP server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

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
        Ok(response) => Json(response).into_response(),
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
) -> Result<JsonRpcResponse> {
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
}
