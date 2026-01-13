use super::IssueSeverity; // Re-use from challenge module
use serde::{Deserialize, Serialize};

/// Verdict from Codex archive quality review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveReviewVerdict {
    /// All checks passed, safe to archive
    Approved,
    /// Minor issues found, needs fixing
    NeedsFix,
    /// Major issues, requires manual intervention
    Rejected,
    /// Could not parse verdict
    Unknown,
}

impl ArchiveReviewVerdict {
    /// Get display name for verdict
    pub fn name(&self) -> &'static str {
        match self {
            ArchiveReviewVerdict::Approved => "APPROVED",
            ArchiveReviewVerdict::NeedsFix => "NEEDS_FIX",
            ArchiveReviewVerdict::Rejected => "REJECTED",
            ArchiveReviewVerdict::Unknown => "UNKNOWN",
        }
    }

    /// Get emoji symbol for verdict
    pub fn emoji(&self) -> &'static str {
        match self {
            ArchiveReviewVerdict::Approved => "✅",
            ArchiveReviewVerdict::NeedsFix => "⚠️",
            ArchiveReviewVerdict::Rejected => "❌",
            ArchiveReviewVerdict::Unknown => "❓",
        }
    }
}

/// Category of issues found during archive review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveIssueCategory {
    /// Missing content from delta
    MissingContent,
    /// Extra content not in delta (hallucination)
    Hallucination,
    /// Format violation
    FormatError,
    /// CHANGELOG inaccuracy
    ChangelogError,
    /// Cross-reference broken
    BrokenReference,
    /// Other issues
    Other,
}

impl ArchiveIssueCategory {
    /// Get display name for category
    pub fn name(&self) -> &'static str {
        match self {
            ArchiveIssueCategory::MissingContent => "Missing Content",
            ArchiveIssueCategory::Hallucination => "Hallucination",
            ArchiveIssueCategory::FormatError => "Format Error",
            ArchiveIssueCategory::ChangelogError => "CHANGELOG Error",
            ArchiveIssueCategory::BrokenReference => "Broken Reference",
            ArchiveIssueCategory::Other => "Other",
        }
    }
}

/// Issue found during archive review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveReviewIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Category of the issue
    pub category: ArchiveIssueCategory,
    /// Spec file where issue was found
    pub spec_file: String,
    /// Description of the issue
    pub description: String,
}

impl ArchiveReviewIssue {
    /// Create a new archive review issue
    pub fn new(
        severity: IssueSeverity,
        category: ArchiveIssueCategory,
        spec_file: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            spec_file: spec_file.into(),
            description: description.into(),
        }
    }

    /// Format issue for display
    pub fn format(&self) -> String {
        format!(
            "[{}] {}: {} - {}",
            self.severity.name(),
            self.spec_file,
            self.category.name(),
            self.description
        )
    }
}

/// Result of archive quality review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveReview {
    /// Overall verdict
    pub verdict: ArchiveReviewVerdict,
    /// List of issues found
    pub issues: Vec<ArchiveReviewIssue>,
    /// Summary of review findings
    pub summary: String,
}

impl ArchiveReview {
    /// Create a new archive review result
    pub fn new(
        verdict: ArchiveReviewVerdict,
        issues: Vec<ArchiveReviewIssue>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            verdict,
            issues,
            summary: summary.into(),
        }
    }

    /// Check if review passed (Approved verdict)
    pub fn passed(&self) -> bool {
        self.verdict == ArchiveReviewVerdict::Approved
    }

    /// Check if review should block archive
    pub fn blocks_archive(&self) -> bool {
        matches!(
            self.verdict,
            ArchiveReviewVerdict::NeedsFix | ArchiveReviewVerdict::Rejected
        )
    }

    /// Count issues by severity
    pub fn count_by_severity(&self, severity: IssueSeverity) -> usize {
        self.issues.iter().filter(|i| i.severity == severity).count()
    }

    /// Format review summary for display
    pub fn format_summary(&self) -> String {
        format!(
            r#"Archive Review: {} {}

Issues:
- High:   {}
- Medium: {}
- Low:    {}

Summary: {}
"#,
            self.verdict.emoji(),
            self.verdict.name(),
            self.count_by_severity(IssueSeverity::High),
            self.count_by_severity(IssueSeverity::Medium),
            self.count_by_severity(IssueSeverity::Low),
            self.summary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_display() {
        assert_eq!(ArchiveReviewVerdict::Approved.name(), "APPROVED");
        assert_eq!(ArchiveReviewVerdict::Approved.emoji(), "✅");
        assert_eq!(ArchiveReviewVerdict::NeedsFix.name(), "NEEDS_FIX");
        assert_eq!(ArchiveReviewVerdict::NeedsFix.emoji(), "⚠️");
    }

    #[test]
    fn test_review_passed() {
        let review = ArchiveReview::new(
            ArchiveReviewVerdict::Approved,
            vec![],
            "All checks passed",
        );
        assert!(review.passed());
        assert!(!review.blocks_archive());
    }

    #[test]
    fn test_review_blocks() {
        let review = ArchiveReview::new(
            ArchiveReviewVerdict::NeedsFix,
            vec![ArchiveReviewIssue::new(
                IssueSeverity::High,
                ArchiveIssueCategory::MissingContent,
                "auth.md",
                "Missing R3",
            )],
            "Issues found",
        );
        assert!(!review.passed());
        assert!(review.blocks_archive());
        assert_eq!(review.count_by_severity(IssueSeverity::High), 1);
    }
}
