//! StateManager - STATE.yaml CRUD operations

use crate::models::frontmatter::{
    ChecksumEntry, LlmCall, State, StatePhase, Telemetry, ValidationEntry, ValidationMode,
    ValidationResult as FrontmatterValidationResult,
};
use crate::parser::frontmatter::calculate_checksum;
use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Files tracked for staleness detection
const TRACKED_FILES: &[&str] = &[
    "proposal.md",
    "tasks.md",
    "CHALLENGE.md",
    "IMPLEMENTATION.md",
    "VERIFICATION.md",
];

/// State manager for a single change
pub struct StateManager {
    change_dir: PathBuf,
    state: State,
    dirty: bool,
}

impl StateManager {
    /// Load or create state for a change
    pub fn load(change_dir: impl Into<PathBuf>) -> Result<Self> {
        let change_dir = change_dir.into();
        let state_path = change_dir.join("STATE.yaml");

        let state = if state_path.exists() {
            let content = std::fs::read_to_string(&state_path)
                .context("Failed to read STATE.yaml")?;
            serde_yaml::from_str(&content).context("Failed to parse STATE.yaml")?
        } else {
            // Extract change_id from directory name
            let change_id = change_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            State {
                change_id,
                schema_version: "2.0".to_string(),
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
                phase: StatePhase::Proposed,
                iteration: 1,
                last_action: None,
                checksums: HashMap::new(),
                validations: Vec::new(),
                telemetry: None,
            }
        };

        Ok(Self {
            change_dir,
            state,
            dirty: false,
        })
    }

    /// Save state to STATE.yaml
    pub fn save(&mut self) -> Result<()> {
        self.state.updated_at = Some(Utc::now());

        let state_path = self.change_dir.join("STATE.yaml");
        let content = serde_yaml::to_string(&self.state)
            .context("Failed to serialize STATE.yaml")?;

        std::fs::write(&state_path, content).context("Failed to write STATE.yaml")?;

        self.dirty = false;
        Ok(())
    }

    /// Save only if dirty
    pub fn save_if_dirty(&mut self) -> Result<()> {
        if self.dirty {
            self.save()?;
        }
        Ok(())
    }

    /// Get current state (read-only)
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Get change directory
    pub fn change_dir(&self) -> &Path {
        &self.change_dir
    }

    // =========================================================================
    // Phase Management
    // =========================================================================

    /// Update the current phase
    pub fn set_phase(&mut self, phase: StatePhase) {
        self.state.phase = phase;
        self.dirty = true;
    }

    /// Get current phase
    pub fn phase(&self) -> &StatePhase {
        &self.state.phase
    }

    /// Increment iteration (for reproposals)
    pub fn increment_iteration(&mut self) {
        self.state.iteration += 1;
        self.dirty = true;
    }

    /// Set last action
    pub fn set_last_action(&mut self, action: impl Into<String>) {
        self.state.last_action = Some(action.into());
        self.dirty = true;
    }

    // =========================================================================
    // Checksum Management
    // =========================================================================

    /// Update checksum for a file
    pub fn update_checksum(&mut self, filename: &str) -> Result<()> {
        let file_path = self.change_dir.join(filename);
        if !file_path.exists() {
            // Remove checksum if file no longer exists
            self.state.checksums.remove(filename);
            self.dirty = true;
            return Ok(());
        }

        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read {}", filename))?;

        let hash = calculate_checksum(&content);
        self.state.checksums.insert(
            filename.to_string(),
            ChecksumEntry {
                hash,
                validated_at: Some(Utc::now()),
            },
        );
        self.dirty = true;

        Ok(())
    }

    /// Update checksums for all tracked files
    pub fn update_all_checksums(&mut self) -> Result<()> {
        for filename in TRACKED_FILES {
            let file_path = self.change_dir.join(filename);
            if file_path.exists() {
                self.update_checksum(filename)?;
            }
        }

        // Also track spec files
        let specs_dir = self.change_dir.join("specs");
        if specs_dir.exists() {
            for entry in walkdir::WalkDir::new(&specs_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            {
                if let Ok(rel_path) = entry.path().strip_prefix(&self.change_dir) {
                    let filename = rel_path.to_string_lossy().to_string();
                    self.update_checksum(&filename)?;
                }
            }
        }

        Ok(())
    }

    /// Check if a file is stale (content changed since last checksum)
    pub fn is_file_stale(&self, filename: &str) -> Result<bool> {
        let file_path = self.change_dir.join(filename);
        if !file_path.exists() {
            return Ok(false);
        }

        let Some(entry) = self.state.checksums.get(filename) else {
            // No recorded checksum = stale (never validated)
            return Ok(true);
        };

        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read {}", filename))?;

        let current_hash = calculate_checksum(&content);
        Ok(entry.hash != current_hash)
    }

    /// Get full staleness report for all tracked files
    pub fn check_staleness(&self) -> Result<StalenessReport> {
        let mut stale_files = Vec::new();
        let mut missing_checksums = Vec::new();
        let mut up_to_date = Vec::new();

        for filename in TRACKED_FILES {
            let file_path = self.change_dir.join(filename);
            if !file_path.exists() {
                continue;
            }

            if !self.state.checksums.contains_key(*filename) {
                missing_checksums.push(filename.to_string());
            } else if self.is_file_stale(filename)? {
                stale_files.push(filename.to_string());
            } else {
                up_to_date.push(filename.to_string());
            }
        }

        // Check spec files
        let specs_dir = self.change_dir.join("specs");
        if specs_dir.exists() {
            for entry in walkdir::WalkDir::new(&specs_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            {
                if let Ok(rel_path) = entry.path().strip_prefix(&self.change_dir) {
                    let filename = rel_path.to_string_lossy().to_string();
                    if !self.state.checksums.contains_key(&filename) {
                        missing_checksums.push(filename);
                    } else if self.is_file_stale(&filename)? {
                        stale_files.push(filename);
                    } else {
                        up_to_date.push(filename);
                    }
                }
            }
        }

        Ok(StalenessReport {
            stale_files,
            missing_checksums,
            up_to_date,
        })
    }

    // =========================================================================
    // Validation History
    // =========================================================================

    /// Record a validation result
    pub fn record_validation(
        &mut self,
        step: impl Into<String>,
        mode: ValidationMode,
        valid: bool,
        high: u32,
        medium: u32,
        low: u32,
        errors: Vec<String>,
        warnings: Vec<String>,
    ) {
        let entry = ValidationEntry {
            step: step.into(),
            timestamp: Some(Utc::now()),
            rules_version: Some("2.0".to_string()),
            rules_hash: None,
            mode: Some(mode),
            result: Some(FrontmatterValidationResult {
                valid,
                high,
                medium,
                low,
                verdict: None,
                issues_parsed: None,
            }),
            errors,
            warnings,
        };

        self.state.validations.push(entry);
        self.dirty = true;
    }

    /// Record a challenge validation with verdict
    pub fn record_challenge_validation(
        &mut self,
        verdict: &str,
        issues_parsed: u32,
        high: u32,
        medium: u32,
        low: u32,
    ) {
        let entry = ValidationEntry {
            step: "validate-challenge".to_string(),
            timestamp: Some(Utc::now()),
            rules_version: Some("2.0".to_string()),
            rules_hash: None,
            mode: Some(ValidationMode::Normal),
            result: Some(FrontmatterValidationResult {
                valid: true,
                high,
                medium,
                low,
                verdict: Some(verdict.to_string()),
                issues_parsed: Some(issues_parsed),
            }),
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        self.state.validations.push(entry);
        self.dirty = true;
    }

    /// Get last validation for a step
    pub fn last_validation(&self, step: &str) -> Option<&ValidationEntry> {
        self.state
            .validations
            .iter()
            .rev()
            .find(|v| v.step == step)
    }

    /// Clear validation history
    pub fn clear_validations(&mut self) {
        self.state.validations.clear();
        self.dirty = true;
    }

    // =========================================================================
    // Telemetry
    // =========================================================================

    /// Record LLM call telemetry
    pub fn record_llm_call(
        &mut self,
        step: &str,
        model: Option<String>,
        tokens_in: Option<u64>,
        tokens_out: Option<u64>,
        duration_ms: Option<u64>,
    ) {
        let call = LlmCall {
            model,
            tokens_in,
            tokens_out,
            duration_ms,
        };

        let telemetry = self.state.telemetry.get_or_insert_with(Telemetry::default);

        match step {
            "proposal" => telemetry.proposal = Some(call),
            "challenge" => telemetry.challenge = Some(call),
            "reproposal" => telemetry.reproposal = Some(call),
            _ => {}
        }

        self.dirty = true;
    }
}

/// Staleness report for a change
#[derive(Debug, Clone)]
pub struct StalenessReport {
    /// Files that have changed since last checksum
    pub stale_files: Vec<String>,
    /// Files without recorded checksums
    pub missing_checksums: Vec<String>,
    /// Files that are up to date
    pub up_to_date: Vec<String>,
}

impl StalenessReport {
    /// Check if any files are stale
    pub fn has_stale(&self) -> bool {
        !self.stale_files.is_empty()
    }

    /// Check if all tracked files have checksums
    pub fn is_complete(&self) -> bool {
        self.missing_checksums.is_empty()
    }

    /// Check if everything is up to date
    pub fn is_fresh(&self) -> bool {
        self.stale_files.is_empty() && self.missing_checksums.is_empty()
    }

    /// Total number of tracked files
    pub fn total_files(&self) -> usize {
        self.stale_files.len() + self.missing_checksums.len() + self.up_to_date.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_change() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join("test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Create proposal.md
        let mut proposal = std::fs::File::create(change_dir.join("proposal.md")).unwrap();
        writeln!(proposal, "# Test Proposal\n\nContent here").unwrap();

        // Create tasks.md
        let mut tasks = std::fs::File::create(change_dir.join("tasks.md")).unwrap();
        writeln!(tasks, "# Tasks\n\n## Task 1").unwrap();

        (temp_dir, change_dir)
    }

    #[test]
    fn test_load_new_state() {
        let (_temp, change_dir) = setup_test_change();

        let manager = StateManager::load(&change_dir).unwrap();

        assert_eq!(manager.state().change_id, "test-change");
        assert_eq!(manager.state().phase, StatePhase::Proposed);
        assert_eq!(manager.state().iteration, 1);
    }

    #[test]
    fn test_save_and_load() {
        let (_temp, change_dir) = setup_test_change();

        // Create and save state
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.set_phase(StatePhase::Challenged);
            manager.set_last_action("challenge-proposal");
            manager.save().unwrap();
        }

        // Load and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            assert_eq!(manager.state().phase, StatePhase::Challenged);
            assert_eq!(
                manager.state().last_action,
                Some("challenge-proposal".to_string())
            );
        }
    }

    #[test]
    fn test_update_checksums() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.update_checksum("proposal.md").unwrap();
        manager.update_checksum("tasks.md").unwrap();

        assert!(manager.state().checksums.contains_key("proposal.md"));
        assert!(manager.state().checksums.contains_key("tasks.md"));
        assert!(manager
            .state()
            .checksums
            .get("proposal.md")
            .unwrap()
            .hash
            .starts_with("sha256:"));
    }

    #[test]
    fn test_staleness_detection() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.update_checksum("proposal.md").unwrap();

        // Should not be stale initially
        assert!(!manager.is_file_stale("proposal.md").unwrap());

        // Modify file
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Modified Proposal\n\nNew content",
        )
        .unwrap();

        // Should now be stale
        assert!(manager.is_file_stale("proposal.md").unwrap());
    }

    #[test]
    fn test_staleness_report() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.update_checksum("proposal.md").unwrap();
        // Don't update tasks.md checksum

        let report = manager.check_staleness().unwrap();

        assert!(report.up_to_date.contains(&"proposal.md".to_string()));
        assert!(report.missing_checksums.contains(&"tasks.md".to_string()));
        assert!(!report.is_fresh());
    }

    #[test]
    fn test_record_validation() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.record_validation(
            "validate-proposal",
            ValidationMode::Normal,
            true,
            0,
            2,
            1,
            vec![],
            vec!["warning 1".to_string()],
        );

        assert_eq!(manager.state().validations.len(), 1);

        let last = manager.last_validation("validate-proposal").unwrap();
        assert_eq!(last.step, "validate-proposal");
        assert!(last.result.as_ref().unwrap().valid);
        assert_eq!(last.result.as_ref().unwrap().medium, 2);
    }

    #[test]
    fn test_phase_transitions() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        assert_eq!(*manager.phase(), StatePhase::Proposed);

        manager.set_phase(StatePhase::Challenged);
        assert_eq!(*manager.phase(), StatePhase::Challenged);

        manager.set_phase(StatePhase::Implementing);
        assert_eq!(*manager.phase(), StatePhase::Implementing);
    }

    #[test]
    fn test_iteration_increment() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        assert_eq!(manager.state().iteration, 1);

        manager.increment_iteration();
        assert_eq!(manager.state().iteration, 2);

        manager.increment_iteration();
        assert_eq!(manager.state().iteration, 3);
    }
}
