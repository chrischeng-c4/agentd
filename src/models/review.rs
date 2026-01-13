use serde::{Deserialize, Serialize};

// Re-export IssueSeverity from challenge module for consistency
pub use super::challenge::IssueSeverity;

/// Verdict from code review process
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewVerdict {
    /// Implementation is approved and ready for merge
    Approved,
    /// Implementation needs changes (HIGH or MEDIUM issues)
    NeedsChanges,
    /// Implementation has major issues (failing tests, critical security)
    MajorIssues,
    /// Unable to determine verdict
    Unknown,
}

/// Individual issue found during code review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Issue title
    pub title: String,
    /// Severity level
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// File path or "multiple files"
    pub location: String,
    /// Description of what's wrong
    pub description: String,
    /// How to fix the issue
    pub recommendation: String,
}

/// Category of review issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueCategory {
    /// Security vulnerability or concern
    Security,
    /// Performance issue or anti-pattern
    Performance,
    /// Code style or formatting issue
    Style,
    /// Required feature is missing
    MissingFeature,
    /// Implementation behaves incorrectly
    WrongBehavior,
    /// Code doesn't match existing patterns
    Consistency,
    /// Test coverage is insufficient
    TestCoverage,
    /// Edge case not handled
    EdgeCase,
}

impl ReviewVerdict {
    /// Check if the verdict allows proceeding to next phase
    pub fn is_approved(&self) -> bool {
        matches!(self, ReviewVerdict::Approved)
    }

    /// Check if automatic refinement should continue
    pub fn needs_refinement(&self) -> bool {
        matches!(self, ReviewVerdict::NeedsChanges)
    }

    /// Check if there are blocking issues
    pub fn has_major_issues(&self) -> bool {
        matches!(self, ReviewVerdict::MajorIssues)
    }
}

impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewVerdict::Approved => write!(f, "APPROVED"),
            ReviewVerdict::NeedsChanges => write!(f, "NEEDS_CHANGES"),
            ReviewVerdict::MajorIssues => write!(f, "MAJOR_ISSUES"),
            ReviewVerdict::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl std::fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueCategory::Security => write!(f, "Security"),
            IssueCategory::Performance => write!(f, "Performance"),
            IssueCategory::Style => write!(f, "Style"),
            IssueCategory::MissingFeature => write!(f, "Missing Feature"),
            IssueCategory::WrongBehavior => write!(f, "Wrong Behavior"),
            IssueCategory::Consistency => write!(f, "Consistency"),
            IssueCategory::TestCoverage => write!(f, "Test Coverage"),
            IssueCategory::EdgeCase => write!(f, "Edge Case"),
        }
    }
}
