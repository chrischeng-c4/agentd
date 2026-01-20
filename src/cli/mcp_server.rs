//! mcp-server CLI subcommand
//!
//! Starts the MCP server for structured proposal generation.
//! The server communicates via JSON-RPC 2.0 over stdio.

use crate::mcp::McpServer;
use crate::Result;

/// Run the MCP server
///
/// # Arguments
///
/// * `tools` - Optional workflow stage to filter tools (plan, challenge, implement, review, archive)
pub async fn run(tools: Option<&str>) -> Result<()> {
    let server = McpServer::new_for_stage(tools)?;
    server.run().await
}
