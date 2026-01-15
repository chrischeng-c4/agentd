// Agentd - Spec-driven Development Orchestrator
// A Rust-powered tool for iterative proposal refinement through AI orchestration

pub mod cli;
pub mod context;
pub mod fillback;
pub mod models;
pub mod orchestrator;
pub mod parser;
pub mod state;
pub mod ui;
pub mod validator;

pub use anyhow::{Context, Result};
pub use colored::Colorize;

// Re-export commonly used types
pub use models::{Challenge, Change, Requirement, Scenario, Verification};
pub use state::{StalenessReport, StateManager};
