//! mcp-server CLI subcommand
//!
//! Starts the MCP server for structured proposal generation.
//! The server communicates via JSON-RPC 2.0 over stdio.

use crate::mcp::McpServer;
use crate::Result;

/// Run the MCP server
pub async fn run() -> Result<()> {
    let server = McpServer::new()?;
    server.run().await
}
