//! Service layer for agentd
//!
//! This module contains the business logic extracted from MCP tools.
//! Services are shared between MCP tools and CLI commands to ensure
//! consistency and avoid code duplication.

pub mod clarifications_service;
pub mod file_service;
pub mod implementation_service;
pub mod knowledge_service;
pub mod proposal_service;
pub mod spec_service;
pub mod tasks_service;

// Re-export commonly used types
pub use clarifications_service::{create_clarifications, CreateClarificationsInput, QuestionAnswer};
pub use file_service::{list_specs, read_file};
pub use implementation_service::{list_changed_files, read_all_requirements};
pub use knowledge_service::{
    list_knowledge, read_knowledge, write_knowledge, write_main_spec, WriteKnowledgeInput,
};
pub use proposal_service::{append_review, create_proposal, CreateProposalInput, ImpactData};
pub use spec_service::{create_spec, CreateSpecInput, RequirementData, ScenarioData};
pub use tasks_service::{create_tasks, CreateTasksInput, FileActionData, TaskData};
