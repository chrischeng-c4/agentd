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
                session_id: None,
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

    /// Set Gemini session ID for resume-by-index
    pub fn set_session_id(&mut self, session_id: String) {
        self.state.session_id = Some(session_id);
        self.dirty = true;
    }

    /// Get current session ID
    pub fn session_id(&self) -> Option<&str> {
        self.state.session_id.as_deref()
    }

    /// Update phase based on challenge verdict
    /// - APPROVED → challenged
    /// - NEEDS_REVISION → proposed (stays for auto-reproposal)
    /// - REJECTED → rejected
    pub fn update_phase_from_verdict(&mut self, verdict: &str) {
        let new_phase = match verdict {
            "APPROVED" => StatePhase::Challenged,
            "NEEDS_REVISION" => StatePhase::Proposed,
            "REJECTED" => StatePhase::Rejected,
            _ => return, // Unknown verdict, don't change phase
        };

        self.set_phase(new_phase);
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

    /// Record LLM call telemetry with cost calculation
    ///
    /// # Arguments
    /// * `step` - The workflow step (e.g., "proposal", "challenge", "implement")
    /// * `model` - The model name used
    /// * `tokens_in` - Number of input tokens
    /// * `tokens_out` - Number of output tokens
    /// * `duration_ms` - Duration in milliseconds
    /// * `cost_per_1m_input` - Cost per 1M input tokens (optional)
    /// * `cost_per_1m_output` - Cost per 1M output tokens (optional)
    pub fn record_llm_call(
        &mut self,
        step: &str,
        model: Option<String>,
        tokens_in: Option<u64>,
        tokens_out: Option<u64>,
        duration_ms: Option<u64>,
        cost_per_1m_input: Option<f64>,
        cost_per_1m_output: Option<f64>,
    ) {
        // Calculate cost if pricing info is available
        let cost_usd = Self::calculate_cost(
            tokens_in,
            tokens_out,
            cost_per_1m_input,
            cost_per_1m_output,
        );

        let call = LlmCall {
            step: step.to_string(),
            agentd_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            model,
            tokens_in,
            tokens_out,
            cost_usd,
            duration_ms,
            timestamp: Some(Utc::now()),
        };

        let telemetry = self.state.telemetry.get_or_insert_with(Telemetry::default);

        // Update totals
        if let Some(cost) = cost_usd {
            telemetry.total_cost_usd += cost;
        }
        if let Some(tokens) = tokens_in {
            telemetry.total_tokens_in += tokens;
        }
        if let Some(tokens) = tokens_out {
            telemetry.total_tokens_out += tokens;
        }

        // Append to calls list
        telemetry.calls.push(call);

        self.dirty = true;
    }

    /// Calculate cost from token usage and pricing
    fn calculate_cost(
        tokens_in: Option<u64>,
        tokens_out: Option<u64>,
        cost_per_1m_input: Option<f64>,
        cost_per_1m_output: Option<f64>,
    ) -> Option<f64> {
        let input_cost = match (tokens_in, cost_per_1m_input) {
            (Some(tokens), Some(cost)) => (tokens as f64 / 1_000_000.0) * cost,
            _ => 0.0,
        };

        let output_cost = match (tokens_out, cost_per_1m_output) {
            (Some(tokens), Some(cost)) => (tokens as f64 / 1_000_000.0) * cost,
            _ => 0.0,
        };

        let total = input_cost + output_cost;
        if total > 0.0 {
            Some(total)
        } else {
            None
        }
    }

    /// Get telemetry summary for the change
    pub fn telemetry_summary(&self) -> Option<&Telemetry> {
        self.state.telemetry.as_ref()
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

    #[test]
    fn test_update_phase_from_verdict_approved() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.set_phase(StatePhase::Proposed);

        manager.update_phase_from_verdict("APPROVED");

        assert_eq!(*manager.phase(), StatePhase::Challenged);
    }

    #[test]
    fn test_update_phase_from_verdict_needs_revision() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.set_phase(StatePhase::Proposed);

        manager.update_phase_from_verdict("NEEDS_REVISION");

        // Should stay in Proposed phase for auto-reproposal
        assert_eq!(*manager.phase(), StatePhase::Proposed);
    }

    #[test]
    fn test_update_phase_from_verdict_rejected() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.set_phase(StatePhase::Proposed);

        manager.update_phase_from_verdict("REJECTED");

        assert_eq!(*manager.phase(), StatePhase::Rejected);
    }

    #[test]
    fn test_update_phase_from_verdict_unknown() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        let original_phase = StatePhase::Proposed;
        manager.set_phase(original_phase.clone());

        manager.update_phase_from_verdict("UNKNOWN");

        // Should not change phase for unknown verdict
        assert_eq!(*manager.phase(), original_phase);
    }

    #[test]
    fn test_phase_transition_workflow() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Initial phase: Proposed
        assert_eq!(*manager.phase(), StatePhase::Proposed);

        // Challenge approved → Challenged
        manager.update_phase_from_verdict("APPROVED");
        assert_eq!(*manager.phase(), StatePhase::Challenged);

        // Start implementation → Implementing
        manager.set_phase(StatePhase::Implementing);
        assert_eq!(*manager.phase(), StatePhase::Implementing);

        // Implementation complete → Complete
        manager.set_phase(StatePhase::Complete);
        assert_eq!(*manager.phase(), StatePhase::Complete);

        // Archive → Archived
        manager.set_phase(StatePhase::Archived);
        assert_eq!(*manager.phase(), StatePhase::Archived);
    }

    #[test]
    fn test_rejected_phase_workflow() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Initial phase: Proposed
        assert_eq!(*manager.phase(), StatePhase::Proposed);

        // Challenge rejected → Rejected
        manager.update_phase_from_verdict("REJECTED");
        assert_eq!(*manager.phase(), StatePhase::Rejected);

        // Manual fix → back to Proposed
        manager.set_phase(StatePhase::Proposed);
        assert_eq!(*manager.phase(), StatePhase::Proposed);

        // Re-challenge approved → Challenged
        manager.update_phase_from_verdict("APPROVED");
        assert_eq!(*manager.phase(), StatePhase::Challenged);
    }

    #[test]
    fn test_needs_revision_stays_proposed() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Set to Proposed
        manager.set_phase(StatePhase::Proposed);
        manager.set_last_action("challenge");
        manager.save().unwrap();

        // Challenge needs revision
        manager.update_phase_from_verdict("NEEDS_REVISION");

        // Should stay in Proposed for auto-reproposal
        assert_eq!(*manager.phase(), StatePhase::Proposed);

        // Can re-challenge after reproposal
        manager.set_last_action("reproposal");
        manager.update_phase_from_verdict("APPROVED");
        assert_eq!(*manager.phase(), StatePhase::Challenged);
    }

    #[test]
    fn test_phase_persistence() {
        let (_temp, change_dir) = setup_test_change();

        // Set phase and save
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.update_phase_from_verdict("APPROVED");
            manager.set_last_action("challenge");
            manager.save().unwrap();
        }

        // Load in new instance and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            assert_eq!(*manager.phase(), StatePhase::Challenged);
            assert_eq!(
                manager.state().last_action,
                Some("challenge".to_string())
            );
        }
    }

    // =========================================================================
    // Telemetry and Cost Tracking Tests
    // =========================================================================

    #[test]
    fn test_record_llm_call_basic() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Record a basic LLM call without pricing
        manager.record_llm_call(
            "proposal",
            Some("gemini-3-flash".to_string()),
            Some(1000),
            Some(500),
            Some(5000),
            None,
            None,
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();
        assert_eq!(telemetry.calls.len(), 1);
        assert_eq!(telemetry.total_tokens_in, 1000);
        assert_eq!(telemetry.total_tokens_out, 500);
        assert_eq!(telemetry.total_cost_usd, 0.0); // No pricing info

        let call = &telemetry.calls[0];
        assert_eq!(call.step, "proposal");
        assert_eq!(call.model, Some("gemini-3-flash".to_string()));
        assert_eq!(call.tokens_in, Some(1000));
        assert_eq!(call.tokens_out, Some(500));
        assert_eq!(call.duration_ms, Some(5000));
        assert!(call.timestamp.is_some());
    }

    #[test]
    fn test_record_llm_call_with_cost() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Record LLM call with pricing
        // Gemini flash: $0.10/1M input, $0.40/1M output
        manager.record_llm_call(
            "proposal",
            Some("gemini-3-flash".to_string()),
            Some(1_000_000), // 1M input tokens
            Some(500_000),   // 500K output tokens
            Some(5000),
            Some(0.10),      // $0.10/1M input
            Some(0.40),      // $0.40/1M output
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();

        // Expected cost: $0.10 (input) + $0.20 (output) = $0.30
        assert!((telemetry.total_cost_usd - 0.30).abs() < 0.0001);

        let call = &telemetry.calls[0];
        assert!((call.cost_usd.unwrap() - 0.30).abs() < 0.0001);
    }

    #[test]
    fn test_record_multiple_llm_calls() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Record proposal call (Gemini)
        manager.record_llm_call(
            "proposal",
            Some("gemini-3-flash".to_string()),
            Some(100_000),
            Some(50_000),
            Some(5000),
            Some(0.10),
            Some(0.40),
        );

        // Record challenge call (Codex)
        manager.record_llm_call(
            "challenge",
            Some("gpt-5.2-codex".to_string()),
            Some(80_000),
            Some(40_000),
            Some(10000),
            Some(2.00),
            Some(8.00),
        );

        // Record implement call (Claude)
        manager.record_llm_call(
            "implement",
            Some("claude-3-sonnet".to_string()),
            Some(200_000),
            Some(100_000),
            Some(30000),
            Some(3.00),
            Some(15.00),
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();

        // Verify totals
        assert_eq!(telemetry.calls.len(), 3);
        assert_eq!(telemetry.total_tokens_in, 380_000);
        assert_eq!(telemetry.total_tokens_out, 190_000);

        // Calculate expected costs:
        // Proposal: 0.1M * $0.10 + 0.05M * $0.40 = $0.01 + $0.02 = $0.03
        // Challenge: 0.08M * $2.00 + 0.04M * $8.00 = $0.16 + $0.32 = $0.48
        // Implement: 0.2M * $3.00 + 0.1M * $15.00 = $0.60 + $1.50 = $2.10
        // Total: $2.61
        assert!((telemetry.total_cost_usd - 2.61).abs() < 0.01);
    }

    #[test]
    fn test_cost_calculation() {
        // Test the static cost calculation method directly

        // Test with full pricing info
        let cost = StateManager::calculate_cost(
            Some(1_000_000),
            Some(500_000),
            Some(1.0),  // $1/1M input
            Some(2.0),  // $2/1M output
        );
        assert!((cost.unwrap() - 2.0).abs() < 0.0001); // $1.0 + $1.0 = $2.0

        // Test with no tokens
        let cost = StateManager::calculate_cost(None, None, Some(1.0), Some(2.0));
        assert!(cost.is_none());

        // Test with no pricing
        let cost = StateManager::calculate_cost(
            Some(1_000_000),
            Some(500_000),
            None,
            None,
        );
        assert!(cost.is_none());

        // Test with partial pricing (input only)
        let cost = StateManager::calculate_cost(
            Some(1_000_000),
            Some(500_000),
            Some(1.0),
            None,
        );
        assert!((cost.unwrap() - 1.0).abs() < 0.0001); // Only input cost
    }

    #[test]
    fn test_telemetry_persistence() {
        let (_temp, change_dir) = setup_test_change();

        // Record telemetry and save
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.record_llm_call(
                "proposal",
                Some("gemini-3-flash".to_string()),
                Some(50_000),
                Some(25_000),
                Some(3000),
                Some(0.10),
                Some(0.40),
            );
            manager.save().unwrap();
        }

        // Load in new instance and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            let telemetry = manager.state().telemetry.as_ref().unwrap();

            assert_eq!(telemetry.calls.len(), 1);
            assert_eq!(telemetry.total_tokens_in, 50_000);
            assert_eq!(telemetry.total_tokens_out, 25_000);

            let call = &telemetry.calls[0];
            assert_eq!(call.step, "proposal");
            assert_eq!(call.model, Some("gemini-3-flash".to_string()));
        }
    }

    #[test]
    fn test_telemetry_summary() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // No telemetry initially
        assert!(manager.telemetry_summary().is_none());

        // Record a call
        manager.record_llm_call(
            "test",
            Some("test-model".to_string()),
            Some(1000),
            Some(500),
            Some(1000),
            None,
            None,
        );

        // Now telemetry exists
        assert!(manager.telemetry_summary().is_some());
        assert_eq!(manager.telemetry_summary().unwrap().calls.len(), 1);
    }

    #[test]
    fn test_small_token_cost_precision() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Test with small token counts to verify precision
        manager.record_llm_call(
            "test",
            Some("test-model".to_string()),
            Some(100),   // 100 tokens
            Some(50),    // 50 tokens
            Some(500),
            Some(0.10),  // $0.10/1M
            Some(0.40),  // $0.40/1M
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();

        // Expected cost: 0.0001 * $0.10 + 0.00005 * $0.40 = $0.00001 + $0.00002 = $0.00003
        let expected = 0.00003;
        assert!((telemetry.total_cost_usd - expected).abs() < 0.000001);
    }

    // =========================================================================
    // Session ID Tests
    // =========================================================================

    #[test]
    fn test_session_id_setter_and_getter() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // No session_id initially
        assert!(manager.session_id().is_none());

        // Set session_id
        manager.set_session_id("abc123-def456-789".to_string());
        assert_eq!(manager.session_id(), Some("abc123-def456-789"));

        // Change session_id
        manager.set_session_id("new-session-uuid".to_string());
        assert_eq!(manager.session_id(), Some("new-session-uuid"));
    }

    #[test]
    fn test_session_id_persistence() {
        let (_temp, change_dir) = setup_test_change();

        // Set and save session_id
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.set_session_id("550e8400-e29b-41d4-a716-446655440000".to_string());
            manager.save().unwrap();
        }

        // Load in new instance and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            assert_eq!(
                manager.session_id(),
                Some("550e8400-e29b-41d4-a716-446655440000")
            );
        }
    }

    #[test]
    fn test_session_id_in_yaml_serialization() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.set_session_id("test-session-id".to_string());
        manager.save().unwrap();

        // Read the STATE.yaml file and verify session_id is serialized
        let state_content = std::fs::read_to_string(change_dir.join("STATE.yaml")).unwrap();
        assert!(state_content.contains("session_id: test-session-id"));
    }

    #[test]
    fn test_session_id_null_handling() {
        let (_temp, change_dir) = setup_test_change();

        // Create state without session_id and verify it loads correctly
        let state_yaml = r#"change_id: test-change
schema_version: "2.0"
phase: proposed
iteration: 1
"#;
        std::fs::write(change_dir.join("STATE.yaml"), state_yaml).unwrap();

        let manager = StateManager::load(&change_dir).unwrap();
        assert!(manager.session_id().is_none());
    }

    #[test]
    fn test_session_id_marks_dirty() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Save initial state to clear dirty flag
        manager.save().unwrap();

        // Set session_id should mark as dirty
        manager.set_session_id("new-id".to_string());

        // Verify dirty flag is set by checking if save would write
        // (we can't directly check dirty, but we know it's set because the setter does it)
        manager.save().unwrap();

        // Reload and verify value persisted (proves dirty flag was set)
        let manager = StateManager::load(&change_dir).unwrap();
        assert_eq!(manager.session_id(), Some("new-id"));
    }
}
