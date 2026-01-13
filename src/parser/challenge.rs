use crate::models::ChallengeVerdict;
use crate::Result;
use std::path::Path;

/// Parse the verdict from a CHALLENGE.md file
/// Looks for checked boxes in the Verdict section
pub fn parse_challenge_verdict(challenge_path: &Path) -> Result<ChallengeVerdict> {
    if !challenge_path.exists() {
        anyhow::bail!("CHALLENGE.md not found at {:?}", challenge_path);
    }

    let content = std::fs::read_to_string(challenge_path)?;

    // Look for checked verdict boxes (case-insensitive)
    let content_lower = content.to_lowercase();

    if content_lower.contains("[x] approved") || content_lower.contains("[✓] approved") {
        Ok(ChallengeVerdict::Approved)
    } else if content_lower.contains("[x] needs_revision")
        || content_lower.contains("[x] needs revision")
        || content_lower.contains("[✓] needs_revision")
        || content_lower.contains("[✓] needs revision")
    {
        Ok(ChallengeVerdict::NeedsRevision)
    } else if content_lower.contains("[x] rejected") || content_lower.contains("[✓] rejected") {
        Ok(ChallengeVerdict::Rejected)
    } else {
        // No verdict checked or could not parse
        Ok(ChallengeVerdict::Unknown)
    }
}

// Challenge parser - placeholder for future detailed parsing
pub struct ChallengeParser;
