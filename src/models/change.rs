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
    /// Implementation in progress
    Implementing,
    /// Verification/testing in progress
    Testing,
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
            ChangePhase::Implementing => "Implementing",
            ChangePhase::Testing => "Testing",
            ChangePhase::Complete => "Complete",
            ChangePhase::Archived => "Archived",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            ChangePhase::Proposed => "📝",
            ChangePhase::Challenged => "🔍",
            ChangePhase::Implementing => "🔨",
            ChangePhase::Testing => "🧪",
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
            created_at: now.clone(),
            updated_at: now,
            deltas: Vec::new(),
            challenge: None,
            verification: None,
        }
    }

    /// Get the path to this change's directory
    pub fn path(&self, project_root: &Path) -> PathBuf {
        project_root.join("specter/changes").join(&self.id)
    }

    /// Get path to proposal.md
    pub fn proposal_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("proposal.md")
    }

    /// Get path to tasks.md
    pub fn tasks_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("tasks.md")
    }

    /// Get path to diagrams.md
    pub fn diagrams_path(&self, project_root: &Path) -> PathBuf {
        self.path(project_root).join("diagrams.md")
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

        let diagrams = self.diagrams_path(project_root);
        if !diagrams.exists() {
            anyhow::bail!("Missing diagrams.md at {:?}", diagrams);
        }

        let specs = self.specs_path(project_root);
        if !specs.exists() {
            anyhow::bail!("Missing specs/ directory at {:?}", specs);
        }

        Ok(())
    }
}

/// Specter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecterConfig {
    /// Project name
    pub project_name: String,

    /// Gemini CLI command (default: "gemini")
    pub gemini_command: String,

    /// Claude CLI command (default: "claude")
    pub claude_command: String,

    /// Codex CLI command (default: "codex")
    pub codex_command: String,

    /// Scripts directory
    pub scripts_dir: PathBuf,

    /// Validation rules for spec files
    #[serde(default)]
    pub validation: ValidationRules,
}

impl Default for SpecterConfig {
    fn default() -> Self {
        Self {
            project_name: "My Project".to_string(),
            gemini_command: "gemini".to_string(),
            claude_command: "claude".to_string(),
            codex_command: "codex".to_string(),
            scripts_dir: PathBuf::from("specter/scripts"),
            validation: ValidationRules::default(),
        }
    }
}

impl SpecterConfig {
    /// Load config from specter/config.toml
    pub fn load(project_root: &Path) -> anyhow::Result<Self> {
        let config_path = project_root.join("specter/config.toml");
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: SpecterConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to specter/config.toml
    pub fn save(&self, project_root: &Path) -> anyhow::Result<()> {
        let config_path = project_root.join("specter/config.toml");
        std::fs::create_dir_all(config_path.parent().unwrap())?;

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
