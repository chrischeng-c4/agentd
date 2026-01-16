/// Extracts the first paragraph under a specified markdown heading.
///
/// # Arguments
/// * `content` - The markdown content to parse
/// * `heading` - The heading name to search for (without the '#' prefix)
///
/// # Returns
/// The first non-empty paragraph under the heading, trimmed and truncated at 80 characters.
/// Returns an empty string if the heading is not found or has no content.
///
/// # Example
/// ```
/// use agentd::parser::extract_heading_section;
/// let content = "## Summary\n\nThis is the summary paragraph.\n\nAnother paragraph.";
/// let summary = extract_heading_section(content, "Summary");
/// assert_eq!(summary, "This is the summary paragraph.");
/// ```
pub fn extract_heading_section(content: &str, heading: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    // Find the heading
    while i < lines.len() {
        let line = lines[i].trim();

        // Check for markdown heading (## Heading or # Heading, etc.)
        if line.starts_with('#') {
            let heading_text = line.trim_start_matches('#').trim();
            if heading_text.eq_ignore_ascii_case(heading) {
                // Found the heading, now extract the first paragraph
                i += 1;

                // Skip empty lines after heading
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Collect lines until we hit an empty line or another heading
                let mut paragraph = String::new();
                while i < lines.len() {
                    let line = lines[i].trim();

                    // Stop at empty line or another heading
                    if line.is_empty() || line.starts_with('#') {
                        break;
                    }

                    if !paragraph.is_empty() {
                        paragraph.push(' ');
                    }
                    paragraph.push_str(line);
                    i += 1;
                }

                // Truncate at 80 characters with ellipsis (Unicode-safe)
                if paragraph.chars().count() > 80 {
                    // Find the byte index for the 77th character
                    let truncate_index = paragraph
                        .char_indices()
                        .nth(77)
                        .map(|(i, _)| i)
                        .unwrap_or(paragraph.len());
                    paragraph.truncate(truncate_index);
                    paragraph.push_str("...");
                }

                return paragraph;
            }
        }
        i += 1;
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_heading_section_basic() {
        let content = "## Summary\n\nThis is the summary paragraph.\n\nAnother paragraph.";
        let result = extract_heading_section(content, "Summary");
        assert_eq!(result, "This is the summary paragraph.");
    }

    #[test]
    fn test_extract_heading_section_missing_heading() {
        let content = "## Overview\n\nSome content here.";
        let result = extract_heading_section(content, "Summary");
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_heading_section_truncation() {
        let content = "## Summary\n\nThis is a very long summary that exceeds eighty characters and should be truncated with an ellipsis at the end.";
        let result = extract_heading_section(content, "Summary");
        assert_eq!(result.len(), 80);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_extract_heading_section_multiline() {
        let content = "## Summary\n\nThis is line one.\nThis is line two.\nThis is line three.";
        let result = extract_heading_section(content, "Summary");
        assert_eq!(result, "This is line one. This is line two. This is line three.");
    }

    #[test]
    fn test_extract_heading_section_case_insensitive() {
        let content = "## SUMMARY\n\nContent here.";
        let result = extract_heading_section(content, "summary");
        assert_eq!(result, "Content here.");
    }

    #[test]
    fn test_extract_heading_section_no_content() {
        let content = "## Summary\n\n## Another Section\n\nSome content.";
        let result = extract_heading_section(content, "Summary");
        assert_eq!(result, "");
    }

    #[test]
    fn test_extract_heading_section_with_extra_whitespace() {
        let content = "##    Summary   \n\n  This is the summary.  \n\nAnother paragraph.";
        let result = extract_heading_section(content, "Summary");
        assert_eq!(result, "This is the summary.");
    }

    #[test]
    fn test_extract_heading_section_unicode_truncation() {
        // Test with Unicode characters that exceed 80 characters
        let content = "## Summary\n\nThis is a summary with emoji 🎉🎉🎉 and Unicode characters like café, naïve, and ñoño that should be truncated safely without panicking even when characters are multi-byte.";
        let result = extract_heading_section(content, "Summary");
        assert!(result.chars().count() <= 80);
        assert!(result.ends_with("..."));
        // Should not panic - this is the key test
    }
}
