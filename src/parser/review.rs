use crate::models::ReviewVerdict;
use crate::Result;
use std::path::Path;

/// Parse review verdict from REVIEW.md
///
/// Looks for checkbox pattern:
/// - [x] APPROVED
/// - [x] NEEDS_CHANGES
/// - [x] MAJOR_ISSUES
pub fn parse_review_verdict(review_path: &Path) -> Result<ReviewVerdict> {
    if !review_path.exists() {
        return Ok(ReviewVerdict::Unknown);
    }

    let content = std::fs::read_to_string(review_path)?;
    let content_lower = content.to_lowercase();

    // Look for checked boxes in verdict section
    if content_lower.contains("[x] approved") {
        Ok(ReviewVerdict::Approved)
    } else if content_lower.contains("[x] needs_changes") {
        Ok(ReviewVerdict::NeedsChanges)
    } else if content_lower.contains("[x] major_issues") {
        Ok(ReviewVerdict::MajorIssues)
    } else {
        Ok(ReviewVerdict::Unknown)
    }
}
