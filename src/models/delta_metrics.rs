use serde::{Deserialize, Serialize};

/// Metrics computed from spec deltas to decide merging strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaMetrics {
    /// Total number of deltas (changes) detected
    pub total_deltas: usize,

    /// Number of added requirements/sections
    pub added_count: usize,

    /// Number of modified requirements/sections
    pub modified_count: usize,

    /// Number of removed requirements/sections
    pub removed_count: usize,

    /// Number of renamed requirements/sections
    pub renamed_count: usize,

    /// Size of existing spec file in bytes (0 if new file)
    pub existing_spec_size: usize,

    /// Size of delta spec file in bytes
    pub delta_spec_size: usize,

    /// Change ratio: delta_size / existing_size (1.0 if new file)
    pub change_ratio: f64,

    /// Number of requirements in existing spec
    pub existing_req_count: usize,

    /// Number of requirements affected by deltas
    pub affected_req_count: usize,

    /// Requirement change ratio: affected / existing (1.0 if new file)
    pub requirement_change_ratio: f64,

    /// Whether new top-level sections were added
    pub has_new_sections: bool,

    /// Whether database schema changes were detected
    pub has_schema_changes: bool,

    /// Whether new API endpoints were added
    pub has_api_changes: bool,
}

impl DeltaMetrics {
    /// Create new delta metrics
    pub fn new() -> Self {
        Self {
            total_deltas: 0,
            added_count: 0,
            modified_count: 0,
            removed_count: 0,
            renamed_count: 0,
            existing_spec_size: 0,
            delta_spec_size: 0,
            change_ratio: 0.0,
            existing_req_count: 0,
            affected_req_count: 0,
            requirement_change_ratio: 0.0,
            has_new_sections: false,
            has_schema_changes: false,
            has_api_changes: false,
        }
    }

    /// Calculate derived metrics after setting raw values
    pub fn calculate_ratios(&mut self) {
        // Calculate change ratio
        if self.existing_spec_size == 0 {
            self.change_ratio = 1.0; // New file
        } else {
            self.change_ratio = self.delta_spec_size as f64 / self.existing_spec_size as f64;
        }

        // Calculate requirement change ratio
        if self.existing_req_count == 0 {
            self.requirement_change_ratio = 1.0; // New file
        } else {
            self.requirement_change_ratio =
                self.affected_req_count as f64 / self.existing_req_count as f64;
        }

        // Calculate total deltas
        self.total_deltas = self.added_count
            + self.modified_count
            + self.removed_count
            + self.renamed_count;
    }

    /// Format metrics for display
    pub fn format_summary(&self) -> String {
        format!(
            r#"Delta Metrics:
  Total deltas: {}
  - Added: {}
  - Modified: {}
  - Removed: {}
  - Renamed: {}

  Size analysis:
  - Existing spec: {} bytes
  - Delta spec: {} bytes
  - Change ratio: {:.1}%

  Requirement analysis:
  - Existing requirements: {}
  - Affected requirements: {}
  - Requirement change ratio: {:.1}%

  Structure changes:
  - New sections: {}
  - Schema changes: {}
  - API changes: {}
"#,
            self.total_deltas,
            self.added_count,
            self.modified_count,
            self.removed_count,
            self.renamed_count,
            self.existing_spec_size,
            self.delta_spec_size,
            self.change_ratio * 100.0,
            self.existing_req_count,
            self.affected_req_count,
            self.requirement_change_ratio * 100.0,
            if self.has_new_sections { "Yes" } else { "No" },
            if self.has_schema_changes { "Yes" } else { "No" },
            if self.has_api_changes { "Yes" } else { "No" }
        )
    }
}

impl Default for DeltaMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Strategy for merging spec deltas back to main specs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergingStrategy {
    /// Rewrite the entire spec file from scratch
    FullRewrite,
    /// Apply only the specified changes, preserve rest
    DifferentialMerge,
    /// Hybrid approach: rewrite affected sections, preserve others
    Hybrid,
}

impl MergingStrategy {
    /// Get display name for the strategy
    pub fn name(&self) -> &'static str {
        match self {
            MergingStrategy::FullRewrite => "Full Rewrite",
            MergingStrategy::DifferentialMerge => "Differential Merge",
            MergingStrategy::Hybrid => "Hybrid",
        }
    }

    /// Get emoji symbol for the strategy
    pub fn emoji(&self) -> &'static str {
        match self {
            MergingStrategy::FullRewrite => "🔄",
            MergingStrategy::DifferentialMerge => "🔧",
            MergingStrategy::Hybrid => "⚡",
        }
    }

    /// Get description of what this strategy does
    pub fn description(&self) -> &'static str {
        match self {
            MergingStrategy::FullRewrite => {
                "Rewrite the entire spec incorporating all changes (best for major changes)"
            }
            MergingStrategy::DifferentialMerge => {
                "Apply only specified changes, preserve existing content (best for localized changes)"
            }
            MergingStrategy::Hybrid => {
                "Rewrite affected sections, preserve unaffected content (balanced approach)"
            }
        }
    }
}

/// Decision result containing strategy and reasoning
#[derive(Debug, Clone)]
pub struct StrategyDecision {
    /// The chosen merging strategy
    pub strategy: MergingStrategy,

    /// Human-readable reason for this decision
    pub reason: String,

    /// Metrics that led to this decision
    pub metrics: DeltaMetrics,
}

impl StrategyDecision {
    /// Create a new strategy decision
    pub fn new(strategy: MergingStrategy, reason: impl Into<String>, metrics: DeltaMetrics) -> Self {
        Self {
            strategy,
            reason: reason.into(),
            metrics,
        }
    }

    /// Format the decision for display
    pub fn format_summary(&self) -> String {
        format!(
            r#"Merging Strategy Decision:
  Strategy: {} {}
  Reason: {}

{}
"#,
            self.strategy.emoji(),
            self.strategy.name(),
            self.reason,
            self.metrics.format_summary()
        )
    }
}

/// Analyze spec deltas and decide optimal merging strategy
pub fn decide_merging_strategy(metrics: &DeltaMetrics) -> StrategyDecision {
    // Strategy 1: Full Rewrite
    // When: New spec file, or major restructuring needed

    if metrics.existing_spec_size == 0 {
        return StrategyDecision::new(
            MergingStrategy::FullRewrite,
            "New spec file",
            metrics.clone(),
        );
    }

    if metrics.added_count > 0
        && metrics.modified_count == 0
        && metrics.removed_count == 0
        && metrics.renamed_count == 0
    {
        return StrategyDecision::new(
            MergingStrategy::FullRewrite,
            "Only additions, clean rewrite is simpler",
            metrics.clone(),
        );
    }

    if metrics.change_ratio >= 0.5 {
        return StrategyDecision::new(
            MergingStrategy::FullRewrite,
            format!(
                "Major changes ({:.0}% change ratio), cleaner to rewrite",
                metrics.change_ratio * 100.0
            ),
            metrics.clone(),
        );
    }

    if metrics.has_new_sections {
        return StrategyDecision::new(
            MergingStrategy::FullRewrite,
            "New sections added, need reorganization",
            metrics.clone(),
        );
    }

    // Strategy 2: Differential Merge
    // When: Localized changes, preserve existing content

    if metrics.change_ratio < 0.3 && metrics.requirement_change_ratio < 0.4 {
        return StrategyDecision::new(
            MergingStrategy::DifferentialMerge,
            format!(
                "Localized changes ({:.0}% change ratio, {:.0}% requirements affected)",
                metrics.change_ratio * 100.0,
                metrics.requirement_change_ratio * 100.0
            ),
            metrics.clone(),
        );
    }

    // Strategy 3: Hybrid (default)
    // Apply full rewrite to affected sections, preserve others

    StrategyDecision::new(
        MergingStrategy::Hybrid,
        format!(
            "Moderate changes ({:.0}% change ratio), rewrite affected sections only",
            metrics.change_ratio * 100.0
        ),
        metrics.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_spec_file() {
        let mut metrics = DeltaMetrics::new();
        metrics.existing_spec_size = 0;
        metrics.delta_spec_size = 1000;
        metrics.added_count = 3;
        metrics.calculate_ratios();

        let decision = decide_merging_strategy(&metrics);
        assert_eq!(decision.strategy, MergingStrategy::FullRewrite);
        assert!(decision.reason.contains("New spec file"));
    }

    #[test]
    fn test_only_additions() {
        let mut metrics = DeltaMetrics::new();
        metrics.existing_spec_size = 5000;
        metrics.delta_spec_size = 1000;
        metrics.added_count = 2;
        metrics.modified_count = 0;
        metrics.removed_count = 0;
        metrics.calculate_ratios();

        let decision = decide_merging_strategy(&metrics);
        assert_eq!(decision.strategy, MergingStrategy::FullRewrite);
        assert!(decision.reason.contains("Only additions"));
    }

    #[test]
    fn test_major_changes() {
        let mut metrics = DeltaMetrics::new();
        metrics.existing_spec_size = 1000;
        metrics.delta_spec_size = 600;
        metrics.added_count = 2;
        metrics.modified_count = 3;
        metrics.calculate_ratios();

        let decision = decide_merging_strategy(&metrics);
        assert_eq!(decision.strategy, MergingStrategy::FullRewrite);
        assert!(decision.reason.contains("Major changes"));
    }

    #[test]
    fn test_localized_changes() {
        let mut metrics = DeltaMetrics::new();
        metrics.existing_spec_size = 10000;
        metrics.delta_spec_size = 1000;
        metrics.existing_req_count = 10;
        metrics.affected_req_count = 2;
        metrics.modified_count = 2;
        metrics.calculate_ratios();

        let decision = decide_merging_strategy(&metrics);
        assert_eq!(decision.strategy, MergingStrategy::DifferentialMerge);
        assert!(decision.reason.contains("Localized changes"));
    }

    #[test]
    fn test_moderate_changes() {
        let mut metrics = DeltaMetrics::new();
        metrics.existing_spec_size = 10000;
        metrics.delta_spec_size = 4000;
        metrics.existing_req_count = 10;
        metrics.affected_req_count = 5;
        metrics.modified_count = 3;
        metrics.added_count = 2;
        metrics.calculate_ratios();

        let decision = decide_merging_strategy(&metrics);
        assert_eq!(decision.strategy, MergingStrategy::Hybrid);
        assert!(decision.reason.contains("Moderate changes"));
    }

    #[test]
    fn test_new_sections() {
        let mut metrics = DeltaMetrics::new();
        metrics.existing_spec_size = 5000;
        metrics.delta_spec_size = 2000;
        metrics.has_new_sections = true;
        metrics.calculate_ratios();

        let decision = decide_merging_strategy(&metrics);
        assert_eq!(decision.strategy, MergingStrategy::FullRewrite);
        assert!(decision.reason.contains("New sections"));
    }
}
