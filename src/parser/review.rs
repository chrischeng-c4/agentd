use crate::models::ReviewVerdict;
use crate::Result;
use std::path::Path;

/// Parse review verdict from REVIEW.md
///
/// Supports multiple formats:
/// 1. Checkbox format: `[x] APPROVED`, `[x] NEEDS_CHANGES`, `[x] MAJOR_ISSUES`
/// 2. Plain text format: `## Verdict\nAPPROVED` or `## Verdict\n\nAPPROVED` (used by Codex)
pub fn parse_review_verdict(review_path: &Path) -> Result<ReviewVerdict> {
    if !review_path.exists() {
        return Ok(ReviewVerdict::Unknown);
    }

    let content = std::fs::read_to_string(review_path)?;
    let content_lower = content.to_lowercase();

    // Normalize whitespace: collapse multiple newlines to single newline after ## verdict
    let normalized = normalize_verdict_section(&content_lower);

    // Handle checkbox format
    if content_lower.contains("[x] approved") {
        return Ok(ReviewVerdict::Approved);
    }
    if content_lower.contains("[x] needs_changes") {
        return Ok(ReviewVerdict::NeedsChanges);
    }
    if content_lower.contains("[x] major_issues") {
        return Ok(ReviewVerdict::MajorIssues);
    }

    // Handle plain text format (with normalized whitespace)
    if normalized.contains("## verdict\napproved") {
        Ok(ReviewVerdict::Approved)
    } else if normalized.contains("## verdict\nneeds_changes") {
        Ok(ReviewVerdict::NeedsChanges)
    } else if normalized.contains("## verdict\nmajor_issues") {
        Ok(ReviewVerdict::MajorIssues)
    } else {
        Ok(ReviewVerdict::Unknown)
    }
}

/// Normalize the verdict section by collapsing whitespace after "## verdict"
fn normalize_verdict_section(content: &str) -> String {
    // Find "## verdict" and normalize whitespace after it
    if let Some(verdict_idx) = content.find("## verdict") {
        let (before, after) = content.split_at(verdict_idx);
        let after_heading = &after[10..]; // Skip "## verdict"

        // Trim leading whitespace/newlines and get the verdict word
        let trimmed = after_heading.trim_start();
        if let Some(newline_idx) = trimmed.find('\n') {
            let verdict_word = &trimmed[..newline_idx].trim();
            return format!("{}## verdict\n{}\n{}", before, verdict_word, &trimmed[newline_idx..]);
        } else {
            return format!("{}## verdict\n{}", before, trimmed.trim());
        }
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_verdict_single_newline() {
        let content = "## verdict\napproved";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\napproved"));
    }

    #[test]
    fn test_normalize_verdict_double_newline() {
        let content = "## verdict\n\nneeds_changes";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\nneeds_changes"));
    }

    #[test]
    fn test_normalize_verdict_multiple_newlines() {
        let content = "## verdict\n\n\nmajor_issues";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\nmajor_issues"));
    }

    #[test]
    fn test_normalize_verdict_with_prefix() {
        let content = "# Review\n\n## verdict\n\napproved\n\n## next steps";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\napproved"));
    }
}
