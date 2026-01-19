use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;

/// Represents an extracted XML block
#[derive(Debug, Clone, PartialEq)]
pub struct XmlBlock {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Mode for updating XML blocks
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpdateMode {
    /// Replace all existing blocks of this tag
    Replace,
    /// Replace only the last block
    ReplaceLatest,
    /// Append new block at the end
    Append,
}

/// Extract all XML blocks of a specific tag from content
///
/// # Arguments
/// * `content` - The content to parse
/// * `tag` - The XML tag name to extract (e.g., "proposal", "review")
///
/// # Returns
/// Vector of XmlBlock structs containing tag, attributes, content, and positions
///
/// # Example
/// ```
/// let content = r#"<review status="approved">Content</review>"#;
/// let blocks = extract_xml_blocks(content, "review")?;
/// assert_eq!(blocks[0].attributes.get("status"), Some(&"approved".to_string()));
/// ```
pub fn extract_xml_blocks(content: &str, tag: &str) -> Result<Vec<XmlBlock>> {
    // Pattern: <tag attr="value" ...>content</tag>
    // Using lazy matching for content to avoid greedy matching across multiple blocks
    let pattern = format!(r"<({})([^>]*)>([\s\S]*?)</{}>", regex::escape(tag), regex::escape(tag));
    let re = Regex::new(&pattern).context("Failed to compile XML regex")?;

    let mut blocks = Vec::new();

    for cap in re.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let tag_name = cap.get(1).unwrap().as_str().to_string();
        let attrs_str = cap.get(2).unwrap().as_str();
        let content_str = cap.get(3).unwrap().as_str().to_string();

        let attributes = parse_xml_attributes(attrs_str)?;

        blocks.push(XmlBlock {
            tag: tag_name,
            attributes,
            content: content_str,
            start_pos: full_match.start(),
            end_pos: full_match.end(),
        });
    }

    Ok(blocks)
}

/// Extract single XML block (first occurrence) of a specific tag
///
/// # Arguments
/// * `content` - The content to parse
/// * `tag` - The XML tag name to extract
///
/// # Returns
/// Option containing the first XmlBlock found, or None if not found
pub fn extract_xml_block(content: &str, tag: &str) -> Result<Option<XmlBlock>> {
    let blocks = extract_xml_blocks(content, tag)?;
    Ok(blocks.into_iter().next())
}

/// Parse XML attributes from an opening tag
///
/// Supports both double and single quotes: key="value" or key='value'
///
/// # Arguments
/// * `tag_line` - The attributes portion of an opening tag (between tag name and >)
///
/// # Returns
/// HashMap of attribute key-value pairs
///
/// # Example
/// ```
/// let attrs = parse_xml_attributes(r#" status="approved" iteration="1" "#)?;
/// assert_eq!(attrs.get("status"), Some(&"approved".to_string()));
/// assert_eq!(attrs.get("iteration"), Some(&"1".to_string()));
/// ```
pub fn parse_xml_attributes(tag_line: &str) -> Result<HashMap<String, String>> {
    let mut attributes = HashMap::new();

    // Pattern: key="value" or key='value'
    let re = Regex::new(r#"(\w+)=["']([^"']*)["']"#).context("Failed to compile attribute regex")?;

    for cap in re.captures_iter(tag_line) {
        let key = cap.get(1).unwrap().as_str().to_string();
        let value = cap.get(2).unwrap().as_str().to_string();
        attributes.insert(key, value);
    }

    Ok(attributes)
}

/// Wrap content in XML tags with optional attributes
///
/// # Arguments
/// * `tag` - The XML tag name
/// * `content` - The content to wrap
/// * `attrs` - HashMap of attributes to include in the opening tag
///
/// # Returns
/// String with content wrapped in XML tags
///
/// # Example
/// ```
/// let mut attrs = HashMap::new();
/// attrs.insert("status".to_string(), "approved".to_string());
/// let xml = wrap_in_xml("review", "Content here", attrs);
/// assert_eq!(xml, r#"<review status="approved">
/// Content here
/// </review>
/// "#);
/// ```
pub fn wrap_in_xml(tag: &str, content: &str, attrs: HashMap<String, String>) -> String {
    let attrs_str = if attrs.is_empty() {
        String::new()
    } else {
        let mut pairs: Vec<String> = attrs
            .iter()
            .map(|(k, v)| format!(r#"{}="{}""#, k, v))
            .collect();
        pairs.sort(); // Sort for consistent output
        format!(" {}", pairs.join(" "))
    };

    format!("<{}{}>\n{}\n</{}>\n", tag, attrs_str, content.trim(), tag)
}

/// Replace or append XML block in content
///
/// # Arguments
/// * `content` - The original content
/// * `tag` - The XML tag to update
/// * `new_block` - The new XML block content (including tags)
/// * `mode` - Update mode (Replace, ReplaceLatest, or Append)
///
/// # Returns
/// Updated content string
///
/// # Example
/// ```
/// let content = "<review>Old</review>";
/// let new_block = "<review>New</review>";
/// let updated = update_xml_blocks(content, "review", new_block, UpdateMode::Replace)?;
/// assert!(updated.contains("New"));
/// assert!(!updated.contains("Old"));
/// ```
pub fn update_xml_blocks(
    content: &str,
    tag: &str,
    new_block: &str,
    mode: UpdateMode,
) -> Result<String> {
    let blocks = extract_xml_blocks(content, tag)?;

    match mode {
        UpdateMode::Replace => {
            // Remove all existing blocks and append new one at the end
            let mut result = content.to_string();

            // Remove blocks in reverse order to maintain positions
            for block in blocks.iter().rev() {
                result.replace_range(block.start_pos..block.end_pos, "");
            }

            // Append new block at the end
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(new_block);

            Ok(result)
        }
        UpdateMode::ReplaceLatest => {
            if let Some(last_block) = blocks.last() {
                // Replace only the last block
                let mut result = content.to_string();
                result.replace_range(last_block.start_pos..last_block.end_pos, new_block);
                Ok(result)
            } else {
                // No blocks found, append
                let mut result = content.to_string();
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
                result.push_str(new_block);
                Ok(result)
            }
        }
        UpdateMode::Append => {
            // Append new block at the end
            let mut result = content.to_string();
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(new_block);
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_block() {
        let content = r#"<review status="approved">
This is a review
</review>"#;

        let blocks = extract_xml_blocks(content, "review").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].tag, "review");
        assert_eq!(blocks[0].attributes.get("status"), Some(&"approved".to_string()));
        assert!(blocks[0].content.contains("This is a review"));
    }

    #[test]
    fn test_extract_multiple_blocks() {
        let content = r#"<review status="needs_revision" iteration="1">
First review
</review>

<review status="approved" iteration="2">
Second review
</review>"#;

        let blocks = extract_xml_blocks(content, "review").unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].attributes.get("iteration"), Some(&"1".to_string()));
        assert_eq!(blocks[1].attributes.get("iteration"), Some(&"2".to_string()));
    }

    #[test]
    fn test_extract_xml_block_first_only() {
        let content = r#"<review>First</review><review>Second</review>"#;
        let block = extract_xml_block(content, "review").unwrap();
        assert!(block.is_some());
        assert!(block.unwrap().content.contains("First"));
    }

    #[test]
    fn test_parse_attributes() {
        let attrs_str = r#" status="approved" iteration="2" reviewer="codex" "#;
        let attrs = parse_xml_attributes(attrs_str).unwrap();

        assert_eq!(attrs.get("status"), Some(&"approved".to_string()));
        assert_eq!(attrs.get("iteration"), Some(&"2".to_string()));
        assert_eq!(attrs.get("reviewer"), Some(&"codex".to_string()));
    }

    #[test]
    fn test_parse_attributes_single_quotes() {
        let attrs_str = r#" status='approved' iteration='2' "#;
        let attrs = parse_xml_attributes(attrs_str).unwrap();

        assert_eq!(attrs.get("status"), Some(&"approved".to_string()));
        assert_eq!(attrs.get("iteration"), Some(&"2".to_string()));
    }

    #[test]
    fn test_wrap_in_xml_no_attrs() {
        let xml = wrap_in_xml("proposal", "Content here", HashMap::new());
        assert_eq!(xml, "<proposal>\nContent here\n</proposal>\n");
    }

    #[test]
    fn test_wrap_in_xml_with_attrs() {
        let mut attrs = HashMap::new();
        attrs.insert("status".to_string(), "approved".to_string());
        attrs.insert("iteration".to_string(), "1".to_string());

        let xml = wrap_in_xml("review", "Content here", attrs);
        assert!(xml.contains(r#"status="approved""#));
        assert!(xml.contains(r#"iteration="1""#));
        assert!(xml.contains("Content here"));
    }

    #[test]
    fn test_update_xml_blocks_replace() {
        let content = r#"<review>Old 1</review>
<review>Old 2</review>"#;

        let new_block = "<review>New</review>";
        let result = update_xml_blocks(content, "review", new_block, UpdateMode::Replace).unwrap();

        assert!(result.contains("New"));
        assert!(!result.contains("Old 1"));
        assert!(!result.contains("Old 2"));
    }

    #[test]
    fn test_update_xml_blocks_replace_latest() {
        let content = r#"<review>First</review>
<review>Second</review>"#;

        let new_block = "<review>Updated</review>";
        let result = update_xml_blocks(content, "review", new_block, UpdateMode::ReplaceLatest).unwrap();

        assert!(result.contains("First"));
        assert!(!result.contains("Second"));
        assert!(result.contains("Updated"));
    }

    #[test]
    fn test_update_xml_blocks_append() {
        let content = "<review>First</review>";
        let new_block = "<review>Second</review>";
        let result = update_xml_blocks(content, "review", new_block, UpdateMode::Append).unwrap();

        assert!(result.contains("First"));
        assert!(result.contains("Second"));
    }

    #[test]
    fn test_extract_blocks_with_multiline_content() {
        let content = r#"<proposal>
## Summary
This is a summary

## Why
Multiple lines
of content
here
</proposal>"#;

        let blocks = extract_xml_blocks(content, "proposal").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].content.contains("## Summary"));
        assert!(blocks[0].content.contains("Multiple lines"));
    }

    #[test]
    fn test_no_blocks_found() {
        let content = "No XML here";
        let blocks = extract_xml_blocks(content, "review").unwrap();
        assert_eq!(blocks.len(), 0);
    }
}
