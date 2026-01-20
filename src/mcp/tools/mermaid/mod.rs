//! Mermaid Diagram Generation Tools
//!
//! Provides MCP tools for generating various types of Mermaid diagrams from structured JSON inputs.
//! Each tool accepts a JSON schema-validated input and generates syntactically correct Mermaid code.

pub mod flowchart;
pub mod sequence;
pub mod class_diagram;
pub mod state_diagram;
pub mod erd;
pub mod mindmap;
pub mod requirement;
pub mod journey;

use super::ToolDefinition;
use crate::Result;
use serde_json::Value;

/// Get all Mermaid tool definitions
pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        flowchart::definition(),
        sequence::definition(),
        class_diagram::definition(),
        state_diagram::definition(),
        erd::definition(),
        mindmap::definition(),
        requirement::definition(),
        journey::definition(),
    ]
}

/// Call a Mermaid tool by name
pub fn call_tool(name: &str, arguments: &Value) -> Result<String> {
    match name {
        "generate_mermaid_flowchart" => flowchart::execute(arguments),
        "generate_mermaid_sequence" => sequence::execute(arguments),
        "generate_mermaid_class" => class_diagram::execute(arguments),
        "generate_mermaid_state" => state_diagram::execute(arguments),
        "generate_mermaid_erd" => erd::execute(arguments),
        "generate_mermaid_mindmap" => mindmap::execute(arguments),
        "generate_mermaid_requirement" => requirement::execute(arguments),
        "generate_mermaid_journey" => journey::execute(arguments),
        _ => anyhow::bail!("Unknown Mermaid tool: {}", name),
    }
}
