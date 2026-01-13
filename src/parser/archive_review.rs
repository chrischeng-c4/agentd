use crate::models::ArchiveReviewVerdict;
use crate::Result;
use std::path::Path;

/// Parse archive review verdict from ARCHIVE_REVIEW.md
///
/// Looks for checked boxes in the status section:
/// - [x] APPROVED → Approved
/// - [x] NEEDS_FIX → NeedsFix
/// - [x] REJECTED → Rejected
pub fn parse_archive_review_verdict(review_path: &Path) -> Result<ArchiveReviewVerdict> {
    if !review_path.exists() {
        return Ok(ArchiveReviewVerdict::Unknown);
    }

    let content = std::fs::read_to_string(review_path)?;
    let content_lower = content.to_lowercase();

    // Look for checked boxes in status or recommendation section
    if content_lower.contains("[x] approved") {
        Ok(ArchiveReviewVerdict::Approved)
    } else if content_lower.contains("[x] needs_fix") || content_lower.contains("[x] needs fix") {
        Ok(ArchiveReviewVerdict::NeedsFix)
    } else if content_lower.contains("[x] rejected") {
        Ok(ArchiveReviewVerdict::Rejected)
    } else {
        // If no checkbox is marked, it's unknown
        Ok(ArchiveReviewVerdict::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_approved() {
        let content = r#"# Archive Quality Review

## Status: [x] APPROVED

All checks passed.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let verdict = parse_archive_review_verdict(file.path()).unwrap();
        assert_eq!(verdict, ArchiveReviewVerdict::Approved);
    }

    #[test]
    fn test_parse_needs_fix() {
        let content = r#"# Archive Quality Review

## Recommendation
- [x] NEEDS_FIX - Fix issues above
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let verdict = parse_archive_review_verdict(file.path()).unwrap();
        assert_eq!(verdict, ArchiveReviewVerdict::NeedsFix);
    }

    #[test]
    fn test_parse_rejected() {
        let content = r#"# Archive Quality Review

## Status: [x] REJECTED

Major issues found.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let verdict = parse_archive_review_verdict(file.path()).unwrap();
        assert_eq!(verdict, ArchiveReviewVerdict::Rejected);
    }

    #[test]
    fn test_parse_unknown() {
        let content = r#"# Archive Quality Review

## Status: [ ] APPROVED | [ ] NEEDS_FIX | [ ] REJECTED

No verdict marked yet.
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let verdict = parse_archive_review_verdict(file.path()).unwrap();
        assert_eq!(verdict, ArchiveReviewVerdict::Unknown);
    }

    #[test]
    fn test_file_not_found() {
        let verdict = parse_archive_review_verdict(Path::new("/nonexistent/path")).unwrap();
        assert_eq!(verdict, ArchiveReviewVerdict::Unknown);
    }
}
