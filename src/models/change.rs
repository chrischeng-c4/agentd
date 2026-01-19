use super::{Challenge, RequirementDelta, ValidationRules, Verification};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Phase of a change
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangePhase {
    /// Proposal generated
    Proposed,
    /// Challenge generated, awaiting review
    Challenged,
    /// Challenge rejected, requires manual intervention
    Rejected,
    /// Implementation in progress
    Implementing,
    /// All tasks complete, ready to archive
    Complete,
    /// Archived
    Archived,
}

impl ChangePhase {
    pub fn name(&self) -> &'static str {
        match self {
            ChangePhase::Proposed => "Proposed",
            ChangePhase::Challenged => "Challenged",
            ChangePhase::Rejected => "Rejected",
            ChangePhase::Implementing => "Implementing",
            ChangePhase::Complete => "Complete",
            ChangePhase::Archived => "Archived",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            ChangePhase::Proposed => "📝",
            ChangePhase::Challenged => "🔍",
            ChangePhase::Rejected => "⛔",
            ChangePhase::Implementing => "🔨",
            ChangePhase::Complete => "✅",
            ChangePhase::Archived => "📦",
        }
    }
}

/// Represents a change proposal with all associated files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Unique identifier (e.g., "add-oauth")
    pub id: String,

    /// Brief description
    pub description: String,

    /// Current phase
    pub phase: ChangePhase,

    /// Task complexity for model selection
    #[serde(default)]
    pub complexity: Complexity,

    /// When this change was created
    pub created_at: String,

    /// When this change was last modified
    pub updated_at: String,

    /// Spec deltas (what requirements are being added/modified/removed)
    pub deltas: Vec<RequirementDelta>,

    /// Challenge report (if challenged)
    pub challenge: Option<Challenge>,

    /// Verification report (if verified)
    pub verification: Option<Verification>,
}

impl Change {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            id: id.into(),
            description: description.into(),
            phase: ChangePhase::Proposed,
            complexity: Complexity::default(),
            created_at: now.clone(),
            updated_at: now,
            deltas: Vec::new(),
            challenge: None,
            verification: None,
        }
    }

    /// Create a new change with specified complexity
    pub fn with_complexity(mut self, complexity: Complexity) -> Self {
        self.complexity = complexity;
        self
    }

    /// Update complexity and timestamp
    pub fn set_complexity(&mut self, complexity: Complexity) {
        self.complexity = complexity;
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    /// Assess complexity based on change directory contents
    /// Returns estimated complexity based on:
    /// - Number of spec files (< 3 = Low, 3-6 = Medium, 7-10 = High, > 10 = Critical)
    /// - Number of tasks (heuristic from tasks.md line count)
    pub fn assess_complexity(&self, project_root: &Path) -> Complexity {
        let change_dir = self.path(project_root);

        // Count spec files
        let specs_dir = change_dir.join("specs");
        let spec_count = if specs_dir.exists() {
            walkdir::WalkDir::new(&specs_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                .count()
        } else {
            0
        };

        // Estimate task count from tasks.md
        let tasks_path = change_dir.join("tasks.md");
        let task_count = if tasks_path.exists() {
            std::fs::read_to_string(&tasks_path)
                .map(|content| {
                    // Count lines starting with "- [ ]" (task checkboxes)
                    content.lines()
                        .filter(|line| line.trim().starts_with("- [ ]") || line.trim().starts_with("- [x]"))
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        };

        // Combined heuristic
        let score = spec_count + task_count;

        match score {
            0..=4 => Complexity::Low,
            5..=10 => Complexity::Medium,
            11..=20 => Complexity::High,
            _ => Complexity::Critical,
        }
    }

    /// Get the path to this change's directory
    pub fn path(&self, project_root: &Path) -> PathBuf {
        project_root.join("agentd/changes").join(&self.id)
    }

    /// Get path to proposal.md
    pub fn proposal_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("proposal.md")
    }

    /// Get path to tasks.md
    pub fn tasks_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("tasks.md")
    }

    /// Get path to specs directory
    pub fn specs_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("specs")
    }

    /// Get path to CHALLENGE.md
    pub fn challenge_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("CHALLENGE.md")
    }

    /// Get path to IMPLEMENTATION.md
    pub fn implementation_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("IMPLEMENTATION.md")
    }

    /// Get path to REVIEW.md
    pub fn review_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("REVIEW.md")
    }

    /// Get path to VERIFICATION.md
    pub fn verification_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("VERIFICATION.md")
    }

    /// Update phase and timestamp
    pub fn update_phase(&mut self, phase: ChangePhase) {
        self.phase = phase;
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    /// Check if all required files exist
    pub fn validate_structure(&self, project_root: &Path) -> anyhow::Result<()> {
        let proposal = self.proposal_path(project_root);
        if !proposal.exists() {
            anyhow::bail!("Missing proposal.md at {:?}", proposal);
        }

        let tasks = self.tasks_path(project_root);
        if !tasks.exists() {
            anyhow::bail!("Missing tasks.md at {:?}", tasks);
        }

        let specs = self.specs_path(project_root);
        if !specs.exists() {
            anyhow::bail!("Missing specs/ directory at {:?}", specs);
        }

        Ok(())
    }
}

// =============================================================================
// Complexity Assessment
// =============================================================================

/// Task complexity level for model selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    /// Simple changes: < 5 files, single component
    Low,
    /// Moderate changes: 5-15 files, module scope
    Medium,
    /// Complex changes: > 15 files, system scope
    High,
    /// Critical changes: architectural, high risk
    Critical,
}

impl Default for Complexity {
    fn default() -> Self {
        Complexity::Medium
    }
}

impl Complexity {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Complexity::Low),
            "medium" => Some(Complexity::Medium),
            "high" => Some(Complexity::High),
            "critical" => Some(Complexity::Critical),
            _ => None,
        }
    }
}

// =============================================================================
// Model Configuration
// =============================================================================

/// Gemini model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiModelConfig {
    /// Model identifier (e.g., "flash", "pro")
    pub id: String,
    /// Full model name (e.g., "gemini-3-flash-preview")
    pub model: String,
    /// Maximum complexity this model handles
    pub complexity: Complexity,
    /// Cost per 1 million input tokens in USD
    #[serde(default)]
    pub cost_per_1m_input: Option<f64>,
    /// Cost per 1 million output tokens in USD
    #[serde(default)]
    pub cost_per_1m_output: Option<f64>,
}

/// Gemini configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// CLI command (default: "gemini")
    #[serde(default = "default_gemini_command")]
    pub command: String,
    /// Available models
    #[serde(default = "default_gemini_models")]
    pub models: Vec<GeminiModelConfig>,
    /// Default model ID
    #[serde(default = "default_gemini_default")]
    pub default: String,
}

fn default_gemini_command() -> String {
    "gemini".to_string()
}

fn default_gemini_models() -> Vec<GeminiModelConfig> {
    vec![
        GeminiModelConfig {
            id: "flash".to_string(),
            model: "gemini-3-flash-preview".to_string(),
            complexity: Complexity::Medium,
            cost_per_1m_input: Some(0.10),
            cost_per_1m_output: Some(0.40),
        },
        GeminiModelConfig {
            id: "pro".to_string(),
            model: "gemini-3-pro-preview".to_string(),
            complexity: Complexity::Critical,
            cost_per_1m_input: Some(1.25),
            cost_per_1m_output: Some(10.00),
        },
    ]
}

fn default_gemini_default() -> String {
    "flash".to_string()
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            command: default_gemini_command(),
            models: default_gemini_models(),
            default: default_gemini_default(),
        }
    }
}

impl GeminiConfig {
    /// Select model based on complexity
    pub fn select_model(&self, complexity: Complexity) -> &GeminiModelConfig {
        // Find the cheapest model that can handle this complexity
        self.models
            .iter()
            .filter(|m| m.complexity as u8 >= complexity as u8)
            .min_by_key(|m| m.complexity as u8)
            .or_else(|| self.models.iter().max_by_key(|m| m.complexity as u8))
            .unwrap_or(&self.models[0])
    }

    /// Get default model
    pub fn default_model(&self) -> &GeminiModelConfig {
        self.models
            .iter()
            .find(|m| m.id == self.default)
            .unwrap_or(&self.models[0])
    }
}

/// Codex model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexModelConfig {
    /// Model identifier (e.g., "fast", "balanced", "deep")
    pub id: String,
    /// Base model name (e.g., "gpt-5.2-codex")
    pub model: String,
    /// Reasoning level (e.g., "low", "medium", "high", "extra high")
    #[serde(default)]
    pub reasoning: Option<String>,
    /// Maximum complexity this model handles
    pub complexity: Complexity,
    /// Cost per 1 million input tokens in USD
    #[serde(default)]
    pub cost_per_1m_input: Option<f64>,
    /// Cost per 1 million output tokens in USD
    #[serde(default)]
    pub cost_per_1m_output: Option<f64>,
}

impl CodexModelConfig {
    /// Generate CLI model argument
    /// e.g., "gpt-5.2-codex high" or "gpt-5.2"
    pub fn to_cli_arg(&self) -> String {
        match &self.reasoning {
            Some(level) => format!("{} {}", self.model, level),
            None => self.model.clone(),
        }
    }
}

/// Codex configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    /// CLI command (default: "codex")
    #[serde(default = "default_codex_command")]
    pub command: String,
    /// Available models
    #[serde(default = "default_codex_models")]
    pub models: Vec<CodexModelConfig>,
    /// Default model ID
    #[serde(default = "default_codex_default")]
    pub default: String,
}

fn default_codex_command() -> String {
    "codex".to_string()
}

fn default_codex_models() -> Vec<CodexModelConfig> {
    vec![
        CodexModelConfig {
            id: "fast".to_string(),
            model: "gpt-5.2-codex".to_string(),
            reasoning: Some("low".to_string()),
            complexity: Complexity::Low,
            cost_per_1m_input: Some(2.00),
            cost_per_1m_output: Some(8.00),
        },
        CodexModelConfig {
            id: "balanced".to_string(),
            model: "gpt-5.2-codex".to_string(),
            reasoning: Some("medium".to_string()),
            complexity: Complexity::Medium,
            cost_per_1m_input: Some(2.00),
            cost_per_1m_output: Some(8.00),
        },
        CodexModelConfig {
            id: "deep".to_string(),
            model: "gpt-5.2-codex".to_string(),
            reasoning: Some("high".to_string()),
            complexity: Complexity::High,
            cost_per_1m_input: Some(2.00),
            cost_per_1m_output: Some(8.00),
        },
        CodexModelConfig {
            id: "max".to_string(),
            model: "gpt-5.2-codex".to_string(),
            reasoning: Some("extra high".to_string()),
            complexity: Complexity::Critical,
            cost_per_1m_input: Some(2.00),
            cost_per_1m_output: Some(8.00),
        },
    ]
}

fn default_codex_default() -> String {
    "balanced".to_string()
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            command: default_codex_command(),
            models: default_codex_models(),
            default: default_codex_default(),
        }
    }
}

impl CodexConfig {
    /// Select model based on complexity
    pub fn select_model(&self, complexity: Complexity) -> &CodexModelConfig {
        self.models
            .iter()
            .filter(|m| m.complexity as u8 >= complexity as u8)
            .min_by_key(|m| m.complexity as u8)
            .or_else(|| self.models.iter().max_by_key(|m| m.complexity as u8))
            .unwrap_or(&self.models[0])
    }

    /// Get default model
    pub fn default_model(&self) -> &CodexModelConfig {
        self.models
            .iter()
            .find(|m| m.id == self.default)
            .unwrap_or(&self.models[0])
    }
}

/// Claude model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeModelConfig {
    /// Model identifier (e.g., "fast", "balanced", "deep")
    pub id: String,
    /// Model name (e.g., "haiku", "sonnet", "opus")
    pub model: String,
    /// Maximum complexity this model handles
    pub complexity: Complexity,
    /// Cost per 1 million input tokens in USD
    #[serde(default)]
    pub cost_per_1m_input: Option<f64>,
    /// Cost per 1 million output tokens in USD
    #[serde(default)]
    pub cost_per_1m_output: Option<f64>,
}

/// Claude configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// CLI command (default: "claude")
    #[serde(default = "default_claude_command")]
    pub command: String,
    /// Available models
    #[serde(default = "default_claude_models")]
    pub models: Vec<ClaudeModelConfig>,
    /// Default model ID
    #[serde(default = "default_claude_default")]
    pub default: String,
}

fn default_claude_command() -> String {
    "claude".to_string()
}

fn default_claude_models() -> Vec<ClaudeModelConfig> {
    vec![
        ClaudeModelConfig {
            id: "fast".to_string(),
            model: "haiku".to_string(),
            complexity: Complexity::Low,
            cost_per_1m_input: Some(0.80),
            cost_per_1m_output: Some(4.00),
        },
        ClaudeModelConfig {
            id: "balanced".to_string(),
            model: "sonnet".to_string(),
            complexity: Complexity::Medium,
            cost_per_1m_input: Some(3.00),
            cost_per_1m_output: Some(15.00),
        },
        ClaudeModelConfig {
            id: "deep".to_string(),
            model: "opus".to_string(),
            complexity: Complexity::Critical,
            cost_per_1m_input: Some(15.00),
            cost_per_1m_output: Some(75.00),
        },
    ]
}

fn default_claude_default() -> String {
    "balanced".to_string()
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            command: default_claude_command(),
            models: default_claude_models(),
            default: default_claude_default(),
        }
    }
}

impl ClaudeConfig {
    /// Select model based on complexity
    pub fn select_model(&self, complexity: Complexity) -> &ClaudeModelConfig {
        self.models
            .iter()
            .filter(|m| m.complexity as u8 >= complexity as u8)
            .min_by_key(|m| m.complexity as u8)
            .or_else(|| self.models.iter().max_by_key(|m| m.complexity as u8))
            .unwrap_or(&self.models[0])
    }

    /// Get default model
    pub fn default_model(&self) -> &ClaudeModelConfig {
        self.models
            .iter()
            .find(|m| m.id == self.default)
            .unwrap_or(&self.models[0])
    }
}

// =============================================================================
// Agentd Configuration
// =============================================================================

/// Workflow iteration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Max iterations for format validation (proposal format fix)
    #[serde(default = "default_format_iterations")]
    pub format_iterations: u32,

    /// Max iterations for planning phase (proposal ↔ challenge)
    #[serde(default = "default_planning_iterations")]
    pub planning_iterations: u32,

    /// Max iterations for implementation phase (implement ↔ review)
    #[serde(default = "default_implementation_iterations")]
    pub implementation_iterations: u32,

    /// Max iterations for archive phase (archive review)
    #[serde(default = "default_archive_iterations")]
    pub archive_iterations: u32,

    /// Number of retries for script execution on transient failures
    #[serde(default = "default_script_retries")]
    pub script_retries: u32,

    /// Delay between retries in seconds
    #[serde(default = "default_retry_delay_secs")]
    pub retry_delay_secs: u64,

    /// Human-in-the-Loop mode: Sequential generation (proposal → specs → tasks) with manual iteration control
    /// If false, uses fully automated mode (auto-reproposal loop on NEEDS_REVISION)
    #[serde(default = "default_human_in_loop", alias = "use_sequential_generation")]
    pub human_in_loop: bool,
}

fn default_format_iterations() -> u32 { 2 }
fn default_planning_iterations() -> u32 { 2 }
fn default_implementation_iterations() -> u32 { 2 }
fn default_archive_iterations() -> u32 { 1 }
fn default_script_retries() -> u32 { 3 }
fn default_retry_delay_secs() -> u64 { 5 }
fn default_human_in_loop() -> bool { true }

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            format_iterations: default_format_iterations(),
            planning_iterations: default_planning_iterations(),
            implementation_iterations: default_implementation_iterations(),
            archive_iterations: default_archive_iterations(),
            script_retries: default_script_retries(),
            retry_delay_secs: default_retry_delay_secs(),
            human_in_loop: default_human_in_loop(),
        }
    }
}

/// Agentd configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentdConfig {
    /// Project name
    pub project_name: String,

    /// Workflow iteration settings
    #[serde(default)]
    pub workflow: WorkflowConfig,

    /// Gemini configuration
    #[serde(default)]
    pub gemini: GeminiConfig,

    /// Codex configuration
    #[serde(default)]
    pub codex: CodexConfig,

    /// Claude configuration
    #[serde(default)]
    pub claude: ClaudeConfig,

    /// Scripts directory (DEPRECATED: no longer used by orchestrators)
    ///
    /// Shell scripts are no longer generated or required. Orchestrators now call
    /// CLI tools directly. This field is kept for backward compatibility only.
    pub scripts_dir: PathBuf,

    /// Validation rules for spec files
    #[serde(default)]
    pub validation: ValidationRules,

    // Legacy fields for backward compatibility (kept for TOML deserialization)
    #[serde(skip_serializing, default)]
    #[allow(dead_code)]
    gemini_command: Option<String>,
    #[serde(skip_serializing, default)]
    #[allow(dead_code)]
    claude_command: Option<String>,
    #[serde(skip_serializing, default)]
    #[allow(dead_code)]
    codex_command: Option<String>,
}

impl Default for AgentdConfig {
    fn default() -> Self {
        Self {
            project_name: "My Project".to_string(),
            workflow: WorkflowConfig::default(),
            gemini: GeminiConfig::default(),
            codex: CodexConfig::default(),
            claude: ClaudeConfig::default(),
            scripts_dir: PathBuf::from("agentd/scripts"),
            validation: ValidationRules::default(),
            gemini_command: None,
            claude_command: None,
            codex_command: None,
        }
    }
}

impl AgentdConfig {
    /// Load config from agentd/config.toml
    pub fn load(project_root: &Path) -> anyhow::Result<Self> {
        let config_path = project_root.join("agentd/config.toml");
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: AgentdConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to agentd/config.toml
    pub fn save(&self, project_root: &Path) -> anyhow::Result<()> {
        let config_path = project_root.join("agentd/config.toml");
        std::fs::create_dir_all(config_path.parent().unwrap())?;

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Resolve scripts_dir to an absolute path.
    ///
    /// If `scripts_dir` is relative, joins it with `project_root`.
    /// If `scripts_dir` is absolute, returns it unchanged.
    pub fn resolve_scripts_dir(&self, project_root: &Path) -> PathBuf {
        if self.scripts_dir.is_absolute() {
            self.scripts_dir.clone()
        } else {
            project_root.join(&self.scripts_dir)
        }
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_resolve_scripts_dir_relative() {
        let config = AgentdConfig {
            scripts_dir: PathBuf::from("agentd/scripts"),
            ..Default::default()
        };

        let project_root = PathBuf::from("/home/user/project");
        let resolved = config.resolve_scripts_dir(&project_root);

        assert_eq!(resolved, PathBuf::from("/home/user/project/agentd/scripts"));
    }

    #[test]
    fn test_resolve_scripts_dir_absolute() {
        let config = AgentdConfig {
            scripts_dir: PathBuf::from("/opt/agentd/scripts"),
            ..Default::default()
        };

        let project_root = PathBuf::from("/home/user/project");
        let resolved = config.resolve_scripts_dir(&project_root);

        // Absolute path should remain unchanged
        assert_eq!(resolved, PathBuf::from("/opt/agentd/scripts"));
    }

    #[test]
    fn test_default_scripts_dir_is_relative() {
        let config = AgentdConfig::default();
        assert!(!config.scripts_dir.is_absolute());
        assert_eq!(config.scripts_dir, PathBuf::from("agentd/scripts"));
    }
}
