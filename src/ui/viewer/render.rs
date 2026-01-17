//! Markdown and YAML rendering for plan viewer
//!
//! Converts plan files to HTML with:
//! - GFM support (tables, strikethrough, task lists)
//! - Stable heading ID injection for annotation targeting
//! - Syntax highlighting preparation for code blocks

use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

/// Allowlist of safe HTML tags for sanitization
const SAFE_TAGS: &[&str] = &[
    "h1", "h2", "h3", "h4", "h5", "h6", "p", "br", "hr", "a", "em", "strong", "code", "pre",
    "blockquote", "ul", "ol", "li", "table", "thead", "tbody", "tr", "th", "td", "del", "s",
    "input", "div", "span", "img", "sup", "sub", "details", "summary",
];

/// Allowlist of safe HTML attributes
const SAFE_ATTRS: &[&str] = &[
    "id", "class", "href", "src", "alt", "title", "type", "checked", "disabled", "colspan",
    "rowspan", "align", "valign", "width", "height", "name", "open",
];

/// Tags whose content should be completely removed
const STRIP_CONTENT_TAGS: &[&str] = &["script", "style", "noscript", "object", "embed", "applet"];

/// Sanitize HTML by removing potentially dangerous tags and attributes
///
/// This removes:
/// - Script tags and their content
/// - Style tags and their content
/// - Event handlers (onclick, onerror, etc.)
/// - javascript: URLs
/// - Unknown tags (allowlist approach)
fn sanitize_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();
    let mut in_tag = false;
    let mut tag_content = String::new();
    let mut skip_until_closing: Option<String> = None;

    while let Some(c) = chars.next() {
        // If we're skipping content until a closing tag
        if let Some(ref skip_tag) = skip_until_closing {
            if c == '<' {
                in_tag = true;
                tag_content.clear();
                tag_content.push(c);
            } else if c == '>' && in_tag {
                tag_content.push(c);
                in_tag = false;

                // Check if this is the closing tag we're looking for
                let inner = tag_content.trim_start_matches('<').trim_end_matches('>');
                if inner.starts_with('/') {
                    let closing_name = inner
                        .trim_start_matches('/')
                        .split(|c: char| c.is_whitespace())
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                    if &closing_name == skip_tag {
                        skip_until_closing = None;
                    }
                }
            } else if in_tag {
                tag_content.push(c);
            }
            // Skip all other content while inside dangerous tag
            continue;
        }

        if c == '<' {
            in_tag = true;
            tag_content.clear();
            tag_content.push(c);
        } else if c == '>' && in_tag {
            tag_content.push(c);
            in_tag = false;

            // Check if this is a dangerous tag that needs content stripping
            let inner = tag_content.trim_start_matches('<').trim_end_matches('>');
            if !inner.starts_with('/') {
                let tag_name = inner
                    .split(|c: char| c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .to_lowercase();

                if STRIP_CONTENT_TAGS.contains(&tag_name.as_str()) {
                    skip_until_closing = Some(tag_name);
                    continue;
                }
            }

            // Parse and sanitize the tag
            if let Some(sanitized) = sanitize_tag(&tag_content) {
                result.push_str(&sanitized);
            }
            // If None, the tag is dropped entirely
        } else if in_tag {
            tag_content.push(c);
        } else {
            result.push(c);
        }
    }

    result
}

/// Sanitize a single HTML tag, returning None if it should be removed
fn sanitize_tag(tag: &str) -> Option<String> {
    // Extract tag name
    let inner = tag.trim_start_matches('<').trim_end_matches('>');
    let is_closing = inner.starts_with('/');
    let inner = inner.trim_start_matches('/');

    // Get the tag name (first word before space or end)
    let tag_name = inner
        .split(|c: char| c.is_whitespace())
        .next()
        .unwrap_or("")
        .to_lowercase();

    // Check if tag is in allowlist
    if !SAFE_TAGS.contains(&tag_name.as_str()) {
        // Drop unsafe tags like <script>, <style>, <iframe>, etc.
        return None;
    }

    if is_closing {
        return Some(format!("</{}>", tag_name));
    }

    // Parse attributes if present
    let rest = inner.strip_prefix(&tag_name).unwrap_or("").trim();
    if rest.is_empty() {
        return Some(format!("<{}>", tag_name));
    }

    // Sanitize attributes
    let mut safe_attrs = Vec::new();
    for attr in parse_attributes(rest) {
        if let Some((name, value)) = attr.split_once('=') {
            let name = name.trim().to_lowercase();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            // Check attribute allowlist
            if !SAFE_ATTRS.contains(&name.as_str()) {
                continue;
            }

            // Check for event handlers
            if name.starts_with("on") {
                continue;
            }

            // Check for unsafe URL schemes (only allow http, https, mailto, and # for anchors)
            if name == "href" || name == "src" {
                let lower_value = value.to_lowercase();
                let trimmed = lower_value.trim();
                // Allow safe schemes: http, https, mailto, relative paths, and anchors
                let is_safe = trimmed.starts_with("http://")
                    || trimmed.starts_with("https://")
                    || trimmed.starts_with("mailto:")
                    || trimmed.starts_with('#')
                    || (!trimmed.contains(':') && !trimmed.is_empty()); // relative paths
                if !is_safe {
                    // Block javascript:, data:, vbscript:, file:, and other schemes
                    continue;
                }
            }

            safe_attrs.push(format!("{}=\"{}\"", name, html_escape(value)));
        } else {
            // Boolean attribute (e.g., checked, disabled)
            let name = attr.trim().to_lowercase();
            if SAFE_ATTRS.contains(&name.as_str()) {
                safe_attrs.push(name);
            }
        }
    }

    if safe_attrs.is_empty() {
        Some(format!("<{}>", tag_name))
    } else {
        Some(format!("<{} {}>", tag_name, safe_attrs.join(" ")))
    }
}

/// Parse HTML attributes from a string
fn parse_attributes(s: &str) -> Vec<String> {
    let mut attrs = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for c in s.chars() {
        if !in_quotes && (c == '"' || c == '\'') {
            in_quotes = true;
            quote_char = c;
            current.push(c);
        } else if in_quotes && c == quote_char {
            in_quotes = false;
            current.push(c);
        } else if !in_quotes && c.is_whitespace() {
            if !current.trim().is_empty() {
                attrs.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push(c);
        }
    }

    if !current.trim().is_empty() {
        attrs.push(current.trim().to_string());
    }

    attrs
}

/// Slugify a heading text into a valid HTML ID
///
/// Algorithm: lowercase, replace non-alphanumeric with hyphens, trim hyphens
///
/// Examples:
/// - "R1: Native Window Rendering" -> "r1-native-window-rendering"
/// - "Hello World!" -> "hello-world"
/// - "Some   Spaces" -> "some-spaces"
pub fn slugify(text: &str) -> String {
    let mut result = String::new();
    let mut prev_was_hyphen = true; // Start true to trim leading hyphens

    for c in text.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            prev_was_hyphen = false;
        } else if !prev_was_hyphen {
            result.push('-');
            prev_was_hyphen = true;
        }
    }

    // Trim trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }

    result
}

/// Render Markdown content to HTML with heading ID injection
///
/// Features:
/// - GFM: tables, strikethrough, task lists
/// - Heading IDs: injected as `id="slugified-heading"`
/// - Code blocks: marked with language class for syntax highlighting
/// - Preserves inline formatting within headings (bold, italic, code, links)
pub fn render_markdown_to_html(markdown: &str) -> String {
    let options = Options::ENABLE_GFM
        | Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;

    let parser = Parser::new_ext(markdown, options);

    // Collect events and inject heading IDs
    let mut events: Vec<Event> = Vec::new();
    let mut heading_text = String::new();  // Plain text for slug generation
    let mut heading_events: Vec<Event> = Vec::new();  // Buffered events to preserve formatting
    let mut in_heading = false;
    let mut current_heading_level: Option<HeadingLevel> = None;
    let mut heading_counts: HashMap<String, usize> = HashMap::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_heading_level = Some(level);
                heading_text.clear();
                heading_events.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let slug = slugify(&heading_text);

                // Handle duplicate slugs by appending a counter
                let count = heading_counts.entry(slug.clone()).or_insert(0);
                let final_slug = if *count == 0 {
                    slug
                } else {
                    format!("{}-{}", slug, count)
                };
                *heading_counts.get_mut(&slugify(&heading_text)).unwrap() += 1;

                // Create new heading with ID, replaying all buffered child events
                if let Some(level) = current_heading_level {
                    events.push(Event::Start(Tag::Heading {
                        level,
                        id: Some(final_slug.clone().into()),
                        classes: vec![],
                        attrs: vec![],
                    }));
                    // Replay all buffered heading content (preserves inline formatting)
                    events.extend(heading_events.drain(..));
                    events.push(Event::End(TagEnd::Heading(level)));
                }

                current_heading_level = None;
            }
            Event::Text(ref text) if in_heading => {
                // Collect plain text for slug generation
                heading_text.push_str(text);
                // Buffer the event to preserve it
                heading_events.push(event);
            }
            Event::Code(ref code) if in_heading => {
                // Collect plain text for slug generation
                heading_text.push_str(code);
                // Buffer the event to preserve inline code
                heading_events.push(event);
            }
            _ if in_heading => {
                // Buffer all other events inside headings (emphasis, links, etc.)
                heading_events.push(event);
            }
            _ => {
                events.push(event);
            }
        }
    }

    // Render to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());

    // Sanitize HTML to remove potential XSS vectors (script tags, event handlers, etc.)
    sanitize_html(&html_output)
}

/// Render YAML content to HTML with syntax highlighting wrapper
///
/// Wraps YAML content in a pre/code block for highlight.js processing
pub fn render_yaml_to_html(yaml: &str) -> String {
    let escaped = html_escape(yaml);
    format!(
        r#"<pre><code class="language-yaml">{}</code></pre>"#,
        escaped
    )
}

/// Escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Render a "file not found" placeholder
pub fn render_not_found_html(filename: &str) -> String {
    format!(
        r#"<div class="not-found">
    <h2>File not found</h2>
    <p>The file <code>{}</code> does not exist.</p>
</div>"#,
        html_escape(filename)
    )
}

/// Wrap content in a full HTML document with styles and scripts
pub fn wrap_in_document(content: &str, title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{}</title>
    <link rel="stylesheet" href="agentd://styles.css">
    <link rel="stylesheet" href="agentd://highlight.min.css">
</head>
<body>
    <div class="content">
        {}
    </div>
    <script src="agentd://highlight.min.js"></script>
    <script src="agentd://mermaid.min.js"></script>
    <script src="agentd://app.js"></script>
</body>
</html>"#,
        html_escape(title),
        content
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("hello world"), "hello-world");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("R1: Native Window Rendering"), "r1-native-window-rendering");
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("Test (with) Parens"), "test-with-parens");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("Some   Spaces"), "some-spaces");
        assert_eq!(slugify("   Leading"), "leading");
        assert_eq!(slugify("Trailing   "), "trailing");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(slugify("Task 1.1"), "task-1-1");
        assert_eq!(slugify("123 ABC"), "123-abc");
    }

    #[test]
    fn test_slugify_unicode() {
        // Unicode letters like é are alphanumeric and kept
        assert_eq!(slugify("Café"), "café");
        // Em dash is not alphanumeric, replaced with hyphen
        assert_eq!(slugify("Hello\u{2014}World"), "hello-world");
    }

    #[test]
    fn test_render_markdown_basic() {
        let md = "# Hello World\n\nThis is a paragraph.";
        let html = render_markdown_to_html(md);

        assert!(html.contains("<h1"));
        assert!(html.contains("id=\"hello-world\""));
        assert!(html.contains("<p>"));
    }

    #[test]
    fn test_render_markdown_multiple_headings() {
        let md = "# First\n\n## Second\n\n### Third";
        let html = render_markdown_to_html(md);

        assert!(html.contains("id=\"first\""));
        assert!(html.contains("id=\"second\""));
        assert!(html.contains("id=\"third\""));
    }

    #[test]
    fn test_render_markdown_duplicate_headings() {
        let md = "# Test\n\n# Test\n\n# Test";
        let html = render_markdown_to_html(md);

        assert!(html.contains("id=\"test\""));
        assert!(html.contains("id=\"test-1\""));
        assert!(html.contains("id=\"test-2\""));
    }

    #[test]
    fn test_render_markdown_gfm_tables() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = render_markdown_to_html(md);

        assert!(html.contains("<table>"));
        assert!(html.contains("<th>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_render_markdown_gfm_strikethrough() {
        let md = "This is ~~deleted~~ text";
        let html = render_markdown_to_html(md);

        assert!(html.contains("<del>"));
    }

    #[test]
    fn test_render_markdown_gfm_tasklist() {
        let md = "- [ ] Unchecked\n- [x] Checked";
        let html = render_markdown_to_html(md);

        assert!(html.contains("type=\"checkbox\""));
    }

    #[test]
    fn test_render_markdown_code_blocks() {
        let md = "```rust\nfn main() {}\n```";
        let html = render_markdown_to_html(md);

        assert!(html.contains("<code"));
        assert!(html.contains("language-rust"));
    }

    #[test]
    fn test_render_markdown_heading_with_inline_code() {
        let md = "# Using the `foo` command";
        let html = render_markdown_to_html(md);

        // Should have heading with ID
        assert!(html.contains("id=\"using-the-foo-command\""));
        // Should preserve inline code formatting
        assert!(html.contains("<code>foo</code>"));
    }

    #[test]
    fn test_render_markdown_heading_with_emphasis() {
        let md = "# This is *important* text";
        let html = render_markdown_to_html(md);

        // Should have heading with ID
        assert!(html.contains("id=\"this-is-important-text\""));
        // Should preserve emphasis formatting
        assert!(html.contains("<em>important</em>"));
    }

    #[test]
    fn test_render_markdown_heading_with_bold() {
        let md = "## **Critical** Section";
        let html = render_markdown_to_html(md);

        // Should have heading with ID
        assert!(html.contains("id=\"critical-section\""));
        // Should preserve bold formatting
        assert!(html.contains("<strong>Critical</strong>"));
    }

    #[test]
    fn test_render_markdown_heading_with_link() {
        let md = "### See the [docs](https://example.com)";
        let html = render_markdown_to_html(md);

        // Should have heading with ID
        assert!(html.contains("id=\"see-the-docs\""));
        // Should preserve link
        assert!(html.contains("<a href=\"https://example.com\">docs</a>"));
    }

    #[test]
    fn test_render_markdown_heading_mixed_formatting() {
        let md = "# The `foo` function is **important**";
        let html = render_markdown_to_html(md);

        // Should have heading with ID
        assert!(html.contains("id=\"the-foo-function-is-important\""));
        // Should preserve all formatting
        assert!(html.contains("<code>foo</code>"));
        assert!(html.contains("<strong>important</strong>"));
    }

    #[test]
    fn test_render_yaml_to_html() {
        let yaml = "key: value\nlist:\n  - item1\n  - item2";
        let html = render_yaml_to_html(yaml);

        assert!(html.contains("<pre>"));
        assert!(html.contains("<code class=\"language-yaml\">"));
        assert!(html.contains("key: value"));
    }

    #[test]
    fn test_render_yaml_escaping() {
        let yaml = "<script>alert('xss')</script>";
        let html = render_yaml_to_html(yaml);

        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn test_render_not_found() {
        let html = render_not_found_html("CHALLENGE.md");

        assert!(html.contains("File not found"));
        assert!(html.contains("CHALLENGE.md"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<>&\"'"), "&lt;&gt;&amp;&quot;&#39;");
    }

    #[test]
    fn test_wrap_in_document() {
        let content = "<h1>Hello</h1>";
        let html = wrap_in_document(content, "Test");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test</title>"));
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("agentd://styles.css"));
        assert!(html.contains("agentd://mermaid.min.js"));
    }

    // HTML Sanitization tests
    #[test]
    fn test_sanitize_removes_script_tags() {
        let input = "<p>Hello</p><script>alert('xss')</script><p>World</p>";
        let result = sanitize_html(input);

        assert!(!result.contains("<script>"));
        assert!(!result.contains("alert"));
        assert!(result.contains("<p>Hello</p>"));
        assert!(result.contains("<p>World</p>"));
    }

    #[test]
    fn test_sanitize_removes_event_handlers() {
        let input = r#"<a href="test" onclick="evil()">Click</a>"#;
        let result = sanitize_html(input);

        assert!(!result.contains("onclick"));
        assert!(result.contains("href=\"test\""));
        assert!(result.contains("<a"));
    }

    #[test]
    fn test_sanitize_removes_javascript_urls() {
        let input = r#"<a href="javascript:alert('xss')">Click</a>"#;
        let result = sanitize_html(input);

        assert!(!result.contains("javascript:"));
    }

    #[test]
    fn test_sanitize_allows_safe_tags() {
        let input = "<h1>Title</h1><p>Para</p><strong>Bold</strong><em>Italic</em>";
        let result = sanitize_html(input);

        assert!(result.contains("<h1>"));
        assert!(result.contains("<p>"));
        assert!(result.contains("<strong>"));
        assert!(result.contains("<em>"));
    }

    #[test]
    fn test_sanitize_removes_iframe() {
        let input = r#"<p>Before</p><iframe src="evil.com"></iframe><p>After</p>"#;
        let result = sanitize_html(input);

        assert!(!result.contains("<iframe"));
        assert!(result.contains("<p>Before</p>"));
        assert!(result.contains("<p>After</p>"));
    }

    #[test]
    fn test_sanitize_removes_style_tags() {
        let input = "<style>body { display: none; }</style><p>Content</p>";
        let result = sanitize_html(input);

        assert!(!result.contains("<style>"));
        assert!(result.contains("<p>Content</p>"));
    }

    #[test]
    fn test_sanitize_preserves_id_and_class() {
        let input = r#"<div id="test" class="foo bar">Content</div>"#;
        let result = sanitize_html(input);

        assert!(result.contains("id=\"test\""));
        assert!(result.contains("class=\"foo bar\""));
    }

    #[test]
    fn test_markdown_with_raw_html_xss() {
        // This is what pulldown-cmark would pass through if not sanitized
        let md = "# Hello\n\n<script>alert('xss')</script>\n\nWorld";
        let html = render_markdown_to_html(md);

        // Script tag should be removed by sanitizer
        assert!(!html.contains("<script>"));
        assert!(!html.contains("alert"));
        assert!(html.contains("World"));
    }

    #[test]
    fn test_markdown_with_img_onerror() {
        let md = r#"<img src="x" onerror="alert('xss')">"#;
        let html = render_markdown_to_html(md);

        // onerror attribute should be removed
        assert!(!html.contains("onerror"));
    }

    #[test]
    fn test_sanitize_blocks_data_url() {
        let input = r#"<a href="data:text/html,<script>alert('xss')</script>">Click</a>"#;
        let result = sanitize_html(input);

        // data: URL should be removed
        assert!(!result.contains("data:"), "data: URL should be blocked. Got: {}", result);
        // Link text should be preserved
        assert!(result.contains("Click"), "Link text missing. Got: {}", result);
    }

    #[test]
    fn test_sanitize_blocks_vbscript_url() {
        let input = r#"<a href="vbscript:msgbox('xss')">Click</a>"#;
        let result = sanitize_html(input);

        // vbscript: URL should be removed
        assert!(!result.contains("vbscript:"));
    }

    #[test]
    fn test_sanitize_blocks_file_url() {
        let input = r#"<a href="file:///etc/passwd">Click</a>"#;
        let result = sanitize_html(input);

        // file: URL should be removed
        assert!(!result.contains("file:"));
    }

    #[test]
    fn test_sanitize_allows_https_urls() {
        let input = r#"<a href="https://example.com">Link</a>"#;
        let result = sanitize_html(input);

        // https URLs should be allowed
        assert!(result.contains("href=\"https://example.com\""));
    }

    #[test]
    fn test_sanitize_allows_http_urls() {
        let input = r#"<a href="http://example.com">Link</a>"#;
        let result = sanitize_html(input);

        // http URLs should be allowed
        assert!(result.contains("href=\"http://example.com\""));
    }

    #[test]
    fn test_sanitize_allows_mailto_urls() {
        let input = r#"<a href="mailto:test@example.com">Email</a>"#;
        let result = sanitize_html(input);

        // mailto URLs should be allowed
        assert!(result.contains("href=\"mailto:test@example.com\""));
    }

    #[test]
    fn test_sanitize_allows_anchor_urls() {
        let input = "<a href=\"#section-1\">Jump</a>";
        let result = sanitize_html(input);

        // Anchor URLs should be allowed
        assert!(result.contains("href=\"#section-1\""));
    }

    #[test]
    fn test_sanitize_allows_relative_urls() {
        let input = r#"<a href="path/to/file.html">Link</a>"#;
        let result = sanitize_html(input);

        // Relative URLs (no scheme) should be allowed
        assert!(result.contains("href=\"path/to/file.html\""));
    }

    #[test]
    fn test_sanitize_blocks_img_data_url() {
        let input = r#"<img src="data:image/svg+xml,<svg onload='alert(1)'></svg>">"#;
        let result = sanitize_html(input);

        // data: URL in src should be removed
        assert!(!result.contains("data:"));
    }
}
