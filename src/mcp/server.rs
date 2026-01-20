//! MCP Server implementation using JSON-RPC 2.0 over stdio
//!
//! Implements the minimal MCP protocol:
//! - `initialize` - Return server info and capabilities
//! - `tools/list` - Return available tool definitions
//! - `tools/call` - Execute a tool and return result

use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};

use super::tools::ToolRegistry;

/// MCP Server for handling JSON-RPC requests over stdio
pub struct McpServer {
    tool_registry: ToolRegistry,
    project_root: std::path::PathBuf,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// JSON-RPC error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;

impl McpServer {
    /// Create a new MCP server with all tools
    pub fn new() -> Result<Self> {
        let project_root = std::env::current_dir()?;
        Ok(Self {
            tool_registry: ToolRegistry::new(),
            project_root,
        })
    }

    /// Create a new MCP server with tools filtered by workflow stage
    ///
    /// # Arguments
    ///
    /// * `stage` - Optional workflow stage (plan, challenge, implement, review, archive)
    ///             If None, all tools are loaded
    pub fn new_for_stage(stage: Option<&str>) -> Result<Self> {
        let project_root = std::env::current_dir()?;
        let tool_registry = match stage {
            Some(s) => {
                eprintln!("[agentd-mcp] Loading tools for stage: {}", s);
                ToolRegistry::new_for_stage(s)
            }
            None => {
                eprintln!("[agentd-mcp] Loading all tools (no stage filter)");
                ToolRegistry::new()
            }
        };

        Ok(Self {
            tool_registry,
            project_root,
        })
    }

    /// Run the MCP server, reading from stdin and writing to stdout
    pub async fn run(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        eprintln!("[agentd-mcp] Server started, waiting for requests...");

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[agentd-mcp] Read error: {}", e);
                    break;
                }
            };

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse and handle request
            let response = self.handle_request(&line).await;

            // Write response
            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        eprintln!("[agentd-mcp] Server stopped");
        Ok(())
    }

    /// Handle a single JSON-RPC request
    async fn handle_request(&self, line: &str) -> JsonRpcResponse {
        // Parse JSON
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
            }
        };

        // Validate jsonrpc version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(Value::Null),
                result: None,
                error: Some(JsonRpcError {
                    code: INVALID_REQUEST,
                    message: "Invalid JSON-RPC version".to_string(),
                    data: None,
                }),
            };
        }

        let id = request.id.clone().unwrap_or(Value::Null);

        // Route to handler
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "initialized" => Ok(json!({})), // Notification, no response needed
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&request.params).await,
            "shutdown" => {
                eprintln!("[agentd-mcp] Shutdown requested");
                Ok(json!({}))
            }
            _ => Err((
                METHOD_NOT_FOUND,
                format!("Method not found: {}", request.method),
            )),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(value),
                error: None,
            },
            Err((code, message)) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code,
                    message,
                    data: None,
                }),
            },
        }
    }

    /// Handle `initialize` request
    fn handle_initialize(&self, _params: &Option<Value>) -> std::result::Result<Value, (i32, String)> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "agentd-mcp",
                "version": env!("CARGO_PKG_VERSION")
            },
            "capabilities": {
                "tools": {}
            }
        }))
    }

    /// Handle `tools/list` request
    fn handle_tools_list(&self) -> std::result::Result<Value, (i32, String)> {
        let tools = self.tool_registry.list_tools();
        Ok(json!({ "tools": tools }))
    }

    /// Handle `tools/call` request
    async fn handle_tools_call(
        &self,
        params: &Option<Value>,
    ) -> std::result::Result<Value, (i32, String)> {
        let params = params
            .as_ref()
            .ok_or((INVALID_PARAMS, "Missing params".to_string()))?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or((INVALID_PARAMS, "Missing tool name".to_string()))?;

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        eprintln!("[agentd-mcp] Calling tool: {} with args: {}", name, arguments);

        // Execute the tool
        match self.tool_registry.call_tool(name, &arguments, &self.project_root).await {
            Ok(result) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": result
                }]
            })),
            Err(e) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", e)
                }],
                "isError": true
            })),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new().expect("Failed to create MCP server")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "initialize");
        assert_eq!(request.jsonrpc, "2.0");
    }

    #[test]
    fn test_serialize_response() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            result: Some(json!({"status": "ok"})),
            error: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }
}
