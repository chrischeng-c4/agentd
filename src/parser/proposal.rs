use crate::Result;
use regex::Regex;

/// Parse affected_specs list from proposal.md
///
/// Looks for the "Affected specs:" line in the Impact section and extracts
/// the list of spec IDs.
///
/// # Supported formats:
/// - Array notation: `["auth-flow", "user-model"]`
/// - Backtick list: `auth-flow`, `user-model`
/// - Comma-separated: auth-flow, user-model
///
/// # Example
/// ```markdown
/// ## Impact
///
/// - Scope: minor
/// - Affected specs: `auth-flow`, `user-model`, `api-endpoints`
/// - Affected code: src/auth/, src/models/
/// ```
///
/// Returns: `vec!["auth-flow", "user-model", "api-endpoints"]`
pub fn parse_affected_specs(content: &str) -> Result<Vec<String>> {
    // Try new XML format first
    if let Ok(Some(proposal_block)) = crate::parser::extract_xml_block(content, "proposal") {
        return parse_affected_specs_from_content(&proposal_block.content);
    }

    // Fallback to old format (backward compatibility)
    parse_affected_specs_from_content(content)
}

/// Parse affected specs from proposal content (XML or plain text)
fn parse_affected_specs_from_content(content: &str) -> Result<Vec<String>> {
    // Look for "Affected specs:" line in Impact section
    // Match patterns like:
    // - Affected specs: ["auth-flow", "user-model"]
    // - Affected specs: `auth-flow`, `user-model`
    // - Affected specs: auth-flow, user-model

    let re = Regex::new(r"(?mi)^[-*]\s*Affected specs:\s*(.+?)$")?;

    if let Some(caps) = re.captures(content) {
        let specs_str = caps.get(1).unwrap().as_str().trim();

        if specs_str.is_empty() || specs_str == "[]" || specs_str == "none" || specs_str.to_lowercase() == "n/a" {
            // No specs required
            return Ok(vec![]);
        }

        // Remove array brackets, backticks, quotes
        let cleaned = specs_str
            .trim_matches(|c| c == '[' || c == ']')
            .replace('`', "")
            .replace('"', "")
            .replace('\'', "");

        // Split by comma and clean up
        let specs: Vec<String> = cleaned
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && *s != "none" && s.to_lowercase() != "n/a")
            .map(|s| s.to_string())
            .collect();

        if specs.is_empty() {
            // If found the line but got empty list after parsing, treat as error
            anyhow::bail!("Affected specs list is empty or invalid in proposal.md. Expected format: `spec-1`, `spec-2`");
        }

        Ok(specs)
    } else {
        anyhow::bail!(
            "Could not find 'Affected specs:' in proposal.md Impact section.\n\
            Expected format:\n\
            ## Impact\n\
            - Affected specs: `spec-1`, `spec-2`, `spec-3`"
        );
    }
}

/// Represents a review block from proposal.md
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewBlock {
    pub status: String,
    pub iteration: u32,
    pub reviewer: String,
    pub content: String,
}

/// Extract latest review from proposal.md
///
/// Parses all `<review>` blocks and returns the one with the highest iteration number.
///
/// # Returns
/// - `Ok(Some(ReviewBlock))` if at least one review is found
/// - `Ok(None)` if no reviews are found
/// - `Err` if parsing fails
pub fn parse_latest_review(content: &str) -> Result<Option<ReviewBlock>> {
    let reviews = crate::parser::extract_xml_blocks(content, "review")?;

    if reviews.is_empty() {
        return Ok(None);
    }

    // Get review with highest iteration number
    let latest = reviews
        .iter()
        .max_by_key(|r| {
            r.attributes
                .get("iteration")
                .and_then(|i| i.parse::<u32>().ok())
                .unwrap_or(0)
        })
        .unwrap();

    Ok(Some(ReviewBlock {
        status: latest
            .attributes
            .get("status")
            .cloned()
            .unwrap_or_default(),
        iteration: latest
            .attributes
            .get("iteration")
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or(1),
        reviewer: latest
            .attributes
            .get("reviewer")
            .cloned()
            .unwrap_or_default(),
        content: latest.content.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_affected_specs_backtick_format() {
        let proposal = r#"
# Change: add-oauth

## Summary
Add OAuth authentication

## Impact

- Scope: minor
- Affected specs: `auth-flow`, `user-model`, `api-endpoints`
- Affected code: src/auth/, src/models/
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec!["auth-flow", "user-model", "api-endpoints"]);
    }

    #[test]
    fn test_parse_affected_specs_array_format() {
        let proposal = r#"
## Impact
- Affected specs: ["auth-flow", "user-model"]
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec!["auth-flow", "user-model"]);
    }

    #[test]
    fn test_parse_affected_specs_plain_format() {
        let proposal = r#"
## Impact
- Affected specs: auth-flow, user-model
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec!["auth-flow", "user-model"]);
    }

    #[test]
    fn test_parse_affected_specs_empty() {
        let proposal = r#"
## Impact
- Affected specs: []
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, Vec::<String>::new());
    }

    #[test]
    fn test_parse_affected_specs_none() {
        let proposal = r#"
## Impact
- Affected specs: none
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, Vec::<String>::new());
    }

    #[test]
    fn test_parse_affected_specs_missing() {
        let proposal = r#"
## Impact
- Scope: minor
- Affected code: src/
"#;

        let result = parse_affected_specs(proposal);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not find"));
    }

    #[test]
    fn test_parse_affected_specs_single() {
        let proposal = r#"
## Impact
- Affected specs: `main-spec`
"#;

        let specs = parse_affected_specs(proposal).unwrap();
        assert_eq!(specs, vec!["main-spec"]);
    }
}
