pub mod annotation;
pub mod archive_review;
pub mod challenge;
pub mod change;
pub mod delta_metrics;
pub mod frontmatter;
pub mod requirement;
pub mod review;
pub mod scenario;
pub mod spec_generation;
pub mod spec_rules;
pub mod validation;
pub mod verification;

pub use archive_review::{
    ArchiveIssueCategory, ArchiveReview, ArchiveReviewIssue, ArchiveReviewVerdict,
};
pub use challenge::{Challenge, ChallengeIssue, ChallengeVerdict, IssueSeverity};
pub use change::{
    AgentdConfig, Change, ChangePhase, ClaudeConfig, ClaudeModelConfig, CodexConfig,
    CodexModelConfig, Complexity, GeminiConfig, GeminiModelConfig,
};
pub use delta_metrics::{decide_merging_strategy, DeltaMetrics, MergingStrategy, StrategyDecision};
pub use frontmatter::{
    // Document frontmatter types
    ChallengeFrontmatter, ChallengeVerdictType, ChecksumEntry, Dependencies, DesignElements,
    ExternalDependency, ImpactAssessment, ImpactScope, IssuesSummary, LayerBreakdown, LayerInfo,
    PriorityBreakdown, ProposalFrontmatter, ProposalStatus, RequirementsSummary, Risk,
    RiskSeverity, SpecFrontmatter, SpecReference, State, StatePhase, TasksFrontmatter,
    TasksSummary, Telemetry, ValidationEntry, ValidationMode,
    // Inline block types
    IssueBlock, IssueLocation, IssueSeverity as FrontmatterIssueSeverity, RequirementBlock,
    RequirementPriority, RequirementStatus, TaskAction, TaskBlock, TaskStatus,
};
pub use requirement::{Requirement, RequirementDelta};
pub use review::{IssueCategory, ReviewIssue, ReviewVerdict};
pub use scenario::Scenario;
pub use spec_generation::{SourceFile, SpecGenerationRequest};
pub use spec_rules::{DocumentType as SpecDocumentType, ScenarioFormat, SpecFormatRules};
pub use validation::{
    DocumentType, ErrorCategory, JsonValidationError, Severity, SeverityMap, ValidationCounts,
    ValidationError, ValidationJsonOutput, ValidationOptions, ValidationResult, ValidationRules,
};
pub use verification::{TestResult, TestStatus, Verification};

pub use annotation::{get_author_name, Annotation, AnnotationError, AnnotationResult, AnnotationStore};
