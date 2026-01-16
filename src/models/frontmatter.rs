//! Frontmatter Types for Agentd Documents
//!
//! Defines the YAML frontmatter structures for:
//! - proposal.md
//! - tasks.md
//! - specs/*.md
//! - CHALLENGE.md
//! - STATE.yaml

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Proposal Frontmatter
// =============================================================================

/// Frontmatter for proposal.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalFrontmatter {
    /// Unique identifier for the change (e.g., "add-oauth-login")
    pub id: String,

    /// Document type (always "proposal")
    #[serde(rename = "type")]
    pub doc_type: String,

    /// Document version (incremented on updates)
    pub version: u32,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Last update timestamp
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,

    /// Author (e.g., "gemini", "user")
    #[serde(default)]
    pub author: Option<String>,

    /// Current status
    #[serde(default)]
    pub status: ProposalStatus,

    /// Iteration count (for reproposals)
    #[serde(default = "default_iteration")]
    pub iteration: u32,

    /// Brief summary of the change
    #[serde(default)]
    pub summary: Option<String>,

    /// Impact assessment
    #[serde(default)]
    pub impact: Option<ImpactAssessment>,

    /// Affected spec files
    #[serde(default)]
    pub affected_specs: Vec<SpecReference>,

    /// Dependencies
    #[serde(default)]
    pub dependencies: Option<Dependencies>,

    /// Risk assessment
    #[serde(default)]
    pub risks: Vec<Risk>,
}

fn default_iteration() -> u32 {
    1
}

/// Proposal status values
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    #[default]
    Proposed,
    Challenged,
    Approved,
    Rejected,
}

/// Impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    /// Impact scope
    pub scope: ImpactScope,
    /// Estimated number of affected files
    #[serde(default)]
    pub affected_files: Option<u32>,
    /// Estimated new files
    #[serde(default)]
    pub new_files: Option<u32>,
}

/// Impact scope levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImpactScope {
    Patch,
    Minor,
    Major,
}

/// Reference to a spec file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecReference {
    /// Spec identifier
    pub id: String,
    /// Path relative to change directory
    pub path: String,
}

/// Dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dependencies {
    /// External dependencies (crates, packages)
    #[serde(default)]
    pub external: Vec<ExternalDependency>,
    /// Internal dependencies (modules, services)
    #[serde(default)]
    pub internal: Vec<String>,
}

/// External dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    /// Package name
    pub name: String,
    /// Version constraint
    #[serde(default)]
    pub version: Option<String>,
}

/// Risk entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    /// Risk severity
    pub severity: RiskSeverity,
    /// Risk category
    pub category: String,
    /// Description
    pub description: String,
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
}

// =============================================================================
// Tasks Frontmatter
// =============================================================================

/// Frontmatter for tasks.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksFrontmatter {
    /// Change identifier
    pub id: String,

    /// Document type (always "tasks")
    #[serde(rename = "type")]
    pub doc_type: String,

    /// Document version
    pub version: u32,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Last update timestamp
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,

    /// Reference to proposal
    #[serde(default)]
    pub proposal_ref: Option<String>,

    /// Task summary statistics
    #[serde(default)]
    pub summary: Option<TasksSummary>,

    /// Layer breakdown
    #[serde(default)]
    pub layers: Option<LayerBreakdown>,
}

/// Task summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TasksSummary {
    pub total: u32,
    #[serde(default)]
    pub completed: u32,
    #[serde(default)]
    pub in_progress: u32,
    #[serde(default)]
    pub blocked: u32,
    #[serde(default)]
    pub pending: u32,
}

/// Layer breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerBreakdown {
    #[serde(default)]
    pub data: Option<LayerInfo>,
    #[serde(default)]
    pub logic: Option<LayerInfo>,
    #[serde(default)]
    pub testing: Option<LayerInfo>,
}

/// Layer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub task_count: u32,
    #[serde(default)]
    pub estimated_files: Option<u32>,
}

// =============================================================================
// Spec Frontmatter
// =============================================================================

/// Frontmatter for specs/*.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFrontmatter {
    /// Spec identifier (e.g., "auth-spec")
    pub id: String,

    /// Document type (always "spec")
    #[serde(rename = "type")]
    pub doc_type: String,

    /// Spec title
    pub title: String,

    /// Document version
    pub version: u32,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Last update timestamp
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,

    /// Parent spec (if hierarchical)
    #[serde(default)]
    pub parent_spec: Option<String>,

    /// Child specs
    #[serde(default)]
    pub child_specs: Vec<String>,

    /// Related specs
    #[serde(default)]
    pub related_specs: Vec<SpecReference>,

    /// Requirements summary
    #[serde(default)]
    pub requirements: Option<RequirementsSummary>,

    /// Design elements present
    #[serde(default)]
    pub design_elements: Option<DesignElements>,
}

/// Requirements summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequirementsSummary {
    pub total: u32,
    #[serde(default)]
    pub ids: Vec<String>,
    #[serde(default)]
    pub by_priority: Option<PriorityBreakdown>,
}

/// Priority breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriorityBreakdown {
    #[serde(default)]
    pub high: u32,
    #[serde(default)]
    pub medium: u32,
    #[serde(default)]
    pub low: u32,
}

/// Design elements flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignElements {
    #[serde(default)]
    pub has_mermaid: bool,
    #[serde(default)]
    pub has_json_schema: bool,
    #[serde(default)]
    pub has_pseudo_code: bool,
    #[serde(default)]
    pub has_api_spec: bool,
}

// =============================================================================
// Challenge Frontmatter
// =============================================================================

/// Frontmatter for CHALLENGE.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeFrontmatter {
    /// Challenge identifier
    pub id: String,

    /// Document type (always "challenge")
    #[serde(rename = "type")]
    pub doc_type: String,

    /// Document version
    pub version: u32,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Challenger (e.g., "codex")
    #[serde(default)]
    pub challenger: Option<String>,

    /// Reference to the change being challenged
    #[serde(default)]
    pub change_ref: Option<String>,

    /// Verdict (machine-readable)
    pub verdict: ChallengeVerdictType,

    /// Reason for verdict
    #[serde(default)]
    pub verdict_reason: Option<String>,

    /// Issues summary
    #[serde(default)]
    pub issues: Option<IssuesSummary>,

    /// Source file checksums (for staleness detection)
    #[serde(default)]
    pub source_checksums: HashMap<String, String>,
}

/// Challenge verdict types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChallengeVerdictType {
    Approved,
    NeedsRevision,
    Rejected,
}

/// Issues summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssuesSummary {
    pub total: u32,
    #[serde(default)]
    pub high: u32,
    #[serde(default)]
    pub medium: u32,
    #[serde(default)]
    pub low: u32,
    #[serde(default)]
    pub by_category: Option<HashMap<String, u32>>,
}

// =============================================================================
// State (STATE.yaml)
// =============================================================================

/// State file for tracking change progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Change identifier
    pub change_id: String,

    /// Schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: String,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Last update timestamp
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,

    /// Current phase
    pub phase: StatePhase,

    /// Iteration count
    #[serde(default = "default_iteration")]
    pub iteration: u32,

    /// Last action performed
    #[serde(default)]
    pub last_action: Option<String>,

    /// File checksums
    #[serde(default)]
    pub checksums: HashMap<String, ChecksumEntry>,

    /// Validation history
    #[serde(default)]
    pub validations: Vec<ValidationEntry>,

    /// LLM telemetry (optional)
    #[serde(default)]
    pub telemetry: Option<Telemetry>,
}

fn default_schema_version() -> String {
    "2.0".to_string()
}

/// State phase values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatePhase {
    Proposed,
    Challenged,
    Rejected,
    Implementing,
    Complete,
    Archived,
}

/// Checksum entry with validation timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumEntry {
    pub hash: String,
    #[serde(default)]
    pub validated_at: Option<DateTime<Utc>>,
}

/// Validation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationEntry {
    pub step: String,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub rules_version: Option<String>,
    #[serde(default)]
    pub rules_hash: Option<String>,
    #[serde(default)]
    pub mode: Option<ValidationMode>,
    #[serde(default)]
    pub result: Option<ValidationResult>,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Validation mode
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationMode {
    #[default]
    Normal,
    Strict,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(default)]
    pub high: u32,
    #[serde(default)]
    pub medium: u32,
    #[serde(default)]
    pub low: u32,
    #[serde(default)]
    pub verdict: Option<String>,
    #[serde(default)]
    pub issues_parsed: Option<u32>,
}

/// LLM telemetry - tracks all LLM calls for a change
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Telemetry {
    /// Collection of all LLM calls made during the change lifecycle
    #[serde(default)]
    pub calls: Vec<LlmCall>,
    /// Total cost in USD across all calls
    #[serde(default)]
    pub total_cost_usd: f64,
    /// Total input tokens across all calls
    #[serde(default)]
    pub total_tokens_in: u64,
    /// Total output tokens across all calls
    #[serde(default)]
    pub total_tokens_out: u64,
}

/// Single LLM call telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCall {
    /// Step/phase name (e.g., "proposal", "challenge", "implement", "review")
    pub step: String,
    /// Agentd version that made this call
    #[serde(default)]
    pub agentd_version: Option<String>,
    /// Model name used
    #[serde(default)]
    pub model: Option<String>,
    /// Input tokens used
    #[serde(default)]
    pub tokens_in: Option<u64>,
    /// Output tokens generated
    #[serde(default)]
    pub tokens_out: Option<u64>,
    /// Cost in USD for this call
    #[serde(default)]
    pub cost_usd: Option<f64>,
    /// Duration in milliseconds
    #[serde(default)]
    pub duration_ms: Option<u64>,
    /// Timestamp when the call was made
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
}

// =============================================================================
// Inline YAML Block Types
// =============================================================================

/// Inline task block (in tasks.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBlock {
    pub id: String,
    pub action: TaskAction,
    #[serde(default)]
    pub status: TaskStatus,
    pub file: String,
    #[serde(default)]
    pub spec_ref: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub estimated_lines: Option<u32>,
}

/// Task action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskAction {
    Create,
    Modify,
    Delete,
    Rename,
}

/// Task status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Blocked,
}

/// Inline requirement block (in specs/*.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementBlock {
    pub id: String,
    pub priority: RequirementPriority,
    #[serde(default)]
    pub status: RequirementStatus,
    #[serde(default)]
    pub scenarios: Option<u32>,
    #[serde(default)]
    pub acceptance_criteria: Option<u32>,
}

/// Requirement priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementPriority {
    High,
    Medium,
    Low,
}

/// Requirement status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementStatus {
    #[default]
    Draft,
    Reviewed,
    Approved,
}

/// Inline issue block (in CHALLENGE.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueBlock {
    pub id: u32,
    pub severity: IssueSeverity,
    pub category: String,
    #[serde(default)]
    pub location: Option<IssueLocation>,
    #[serde(default)]
    pub affects_requirements: Vec<String>,
    #[serde(default)]
    pub auto_fixable: Option<bool>,
}

/// Issue severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    High,
    Medium,
    Low,
}

/// Issue location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLocation {
    pub file: String,
    #[serde(default)]
    pub line: Option<u32>,
    #[serde(default)]
    pub section: Option<String>,
}

// =============================================================================
// Default Implementations
// =============================================================================

impl Default for ProposalFrontmatter {
    fn default() -> Self {
        Self {
            id: String::new(),
            doc_type: "proposal".to_string(),
            version: 1,
            created_at: None,
            updated_at: None,
            author: None,
            status: ProposalStatus::default(),
            iteration: 1,
            summary: None,
            impact: None,
            affected_specs: Vec::new(),
            dependencies: None,
            risks: Vec::new(),
        }
    }
}

impl Default for TasksFrontmatter {
    fn default() -> Self {
        Self {
            id: String::new(),
            doc_type: "tasks".to_string(),
            version: 1,
            created_at: None,
            updated_at: None,
            proposal_ref: None,
            summary: None,
            layers: None,
        }
    }
}

impl Default for SpecFrontmatter {
    fn default() -> Self {
        Self {
            id: String::new(),
            doc_type: "spec".to_string(),
            title: String::new(),
            version: 1,
            created_at: None,
            updated_at: None,
            parent_spec: None,
            child_specs: Vec::new(),
            related_specs: Vec::new(),
            requirements: None,
            design_elements: None,
        }
    }
}

impl Default for ChallengeFrontmatter {
    fn default() -> Self {
        Self {
            id: String::new(),
            doc_type: "challenge".to_string(),
            version: 1,
            created_at: None,
            challenger: None,
            change_ref: None,
            verdict: ChallengeVerdictType::NeedsRevision,
            verdict_reason: None,
            issues: None,
            source_checksums: HashMap::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            change_id: String::new(),
            schema_version: default_schema_version(),
            created_at: None,
            updated_at: None,
            phase: StatePhase::Proposed,
            iteration: 1,
            last_action: None,
            checksums: HashMap::new(),
            validations: Vec::new(),
            telemetry: None,
        }
    }
}
