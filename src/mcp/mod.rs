//! MCP (Model Context Protocol) Server for Agentd
//!
//! Provides structured tools for creating proposal files, reducing format errors
//! from free-form LLM output.
//!
//! ## Tools
//! - `create_proposal` - Create proposal.md with enforced structure
//! - `create_spec` - Create specs/*.md with requirements and scenarios
//! - `create_tasks` - Create tasks.md with layered task structure
//! - `validate_change` - Validate all proposal files

pub mod config;
pub mod server;
pub mod tools;

pub use config::{ensure_codex_mcp_config, ensure_gemini_mcp_config};
pub use server::McpServer;
