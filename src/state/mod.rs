//! STATE.yaml Management Module
//!
//! Handles persistence and tracking of change state, including:
//! - Phase transitions
//! - File checksums for staleness detection
//! - Validation history
//! - LLM telemetry

mod manager;

pub use manager::{StateManager, StalenessReport};
