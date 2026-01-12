// Specter - Spec-driven Development Orchestrator
// A Rust-powered tool for iterative proposal refinement through AI orchestration

pub mod cli;
pub mod orchestrator;
pub mod parser;
pub mod validator;
pub mod models;
pub mod ui;

pub use anyhow::{Result, Context};
pub use colored::Colorize;

// Re-export commonly used types
pub use models::{Change, Requirement, Scenario, Challenge, Verification};
