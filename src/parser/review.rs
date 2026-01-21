use crate::models::ReviewVerdict;
use crate::Result;
use std::path::Path;

/// Parse review verdict from REVIEW.md
///
/// Supports two formats:
/// 1. Checkbox format: `[x] APPROVED`, `[x] NEEDS_CHANGES`, `[x] MAJOR_ISSUES`
/// 2. Plain text format: `## Verdict\nAPPROVED` (used by Codex)
pub fn parse_review_verdict(review_path: &Path) -> Result<ReviewVerdict> {
    if !review_path.exists() {
        return Ok(ReviewVerdict::Unknown);
    }

    let content = std::fs::read_to_string(review_path)?;
    let content_lower = content.to_lowercase();

    // Handle both checkbox format and plain text format (## Verdict\nVERDICT_VALUE)
    if content_lower.contains("[x] approved") || content_lower.contains("## verdict\napproved") {
        Ok(ReviewVerdict::Approved)
    } else if content_lower.contains("[x] needs_changes") || content_lower.contains("## verdict\nneeds_changes") {
        Ok(ReviewVerdict::NeedsChanges)
    } else if content_lower.contains("[x] major_issues") || content_lower.contains("## verdict\nmajor_issues") {
        Ok(ReviewVerdict::MajorIssues)
    } else {
        Ok(ReviewVerdict::Unknown)
    }
}
