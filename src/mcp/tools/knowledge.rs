//! Knowledge MCP Tools
//!
//! Tools for reading, writing, and listing knowledge documentation.

use super::{get_optional_string, get_required_string, ToolDefinition};
use crate::Result;
use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;

/// Get the tool definition for read_knowledge
pub fn read_definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_knowledge".to_string(),
        description: "Read a knowledge document. Path is relative to agentd/knowledge/".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path relative to agentd/knowledge/, e.g. 'index.md' or '00-architecture/01-overview.md'"
                }
            }
        }),
    }
}

/// Execute the read_knowledge tool
pub fn execute_read(args: &Value, project_root: &Path) -> Result<String> {
    let path = get_optional_string(args, "path").unwrap_or_else(|| "index.md".to_string());

    let knowledge_dir = project_root.join("agentd/knowledge");
    if !knowledge_dir.exists() {
        anyhow::bail!("Knowledge directory not found. Run 'agentd init' first.");
    }

    // Normalize path and prevent directory traversal
    let normalized_path = path.trim_start_matches('/').trim_start_matches("./");
    if normalized_path.contains("..") {
        anyhow::bail!("Invalid path: directory traversal not allowed");
    }

    let file_path = knowledge_dir.join(normalized_path);

    // Security: ensure path is within knowledge directory
    if !file_path.starts_with(&knowledge_dir) {
        anyhow::bail!("Invalid path: must be within agentd/knowledge/");
    }

    if !file_path.exists() {
        anyhow::bail!("Knowledge file not found: {}", normalized_path);
    }

    if file_path.is_dir() {
        // If directory, try to read index.md
        let index_path = file_path.join("index.md");
        if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)?;
            return Ok(format!(
                "# Knowledge: {}/index.md\n\n{}",
                normalized_path, content
            ));
        } else {
            anyhow::bail!(
                "'{}' is a directory. Specify a file or ensure index.md exists.",
                normalized_path
            );
        }
    }

    let content = std::fs::read_to_string(&file_path)?;

    Ok(format!("# Knowledge: {}\n\n{}", normalized_path, content))
}

/// Get the tool definition for list_knowledge
pub fn list_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_knowledge".to_string(),
        description: "List all knowledge documents with their first line as description"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Optional subdirectory to list, e.g. '00-architecture'. Lists all if not specified."
                }
            }
        }),
    }
}

/// Get the tool definition for write_knowledge
pub fn write_definition() -> ToolDefinition {
    ToolDefinition {
        name: "write_knowledge".to_string(),
        description: "Write or update a knowledge document with auto-generated frontmatter"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["path", "title", "source", "content"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path relative to agentd/knowledge/, e.g. '30-claude/skills.md'"
                },
                "title": {
                    "type": "string",
                    "description": "Document title"
                },
                "source": {
                    "type": "string",
                    "description": "Source URL or description"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content (without frontmatter)"
                }
            }
        }),
    }
}

/// Execute the write_knowledge tool
pub fn execute_write(args: &Value, project_root: &Path) -> Result<String> {
    let path = get_required_string(args, "path")?;
    let title = get_required_string(args, "title")?;
    let source = get_required_string(args, "source")?;
    let content = get_required_string(args, "content")?;

    let knowledge_dir = project_root.join("agentd/knowledge");
    if !knowledge_dir.exists() {
        std::fs::create_dir_all(&knowledge_dir)?;
    }

    // Normalize path and prevent directory traversal
    let normalized_path = path.trim_start_matches('/').trim_start_matches("./");
    if normalized_path.contains("..") {
        anyhow::bail!("Invalid path: directory traversal not allowed");
    }

    let file_path = knowledge_dir.join(normalized_path);

    // Security: ensure path is within knowledge directory
    if !file_path.starts_with(&knowledge_dir) {
        anyhow::bail!("Invalid path: must be within agentd/knowledge/");
    }

    // Create parent directories if needed
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let today = Local::now().format("%Y-%m-%d").to_string();

    // Check if file exists to determine if this is an update
    let is_update = file_path.exists();
    let original_date = if is_update {
        // Try to extract original date from existing frontmatter
        extract_frontmatter_field(&file_path, "date").unwrap_or_else(|| today.clone())
    } else {
        today.clone()
    };

    // Generate frontmatter
    let frontmatter = if is_update {
        format!(
            "---\ntitle: {}\nsource: {}\ndate: {}\nupdated: {}\n---",
            title, source, original_date, today
        )
    } else {
        format!(
            "---\ntitle: {}\nsource: {}\ndate: {}\n---",
            title, source, today
        )
    };

    // Combine frontmatter and content
    let full_content = format!("{}\n\n{}", frontmatter, content.trim());

    std::fs::write(&file_path, full_content)?;

    let action = if is_update { "updated" } else { "written" };
    let mut result = format!("✓ Knowledge {}: {}\n", action, normalized_path);
    result.push_str(&format!("  Title: {}\n", title));
    result.push_str(&format!("  Source: {}\n", source));
    result.push_str(&format!("  Date: {}", today));

    Ok(result)
}

/// Extract a field value from YAML frontmatter
fn extract_frontmatter_field(path: &Path, field: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.first()? != &"---" {
        return None;
    }

    for line in lines.iter().skip(1) {
        if *line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix(&format!("{}: ", field)) {
            return Some(value.to_string());
        }
    }

    None
}

/// Knowledge file metadata
struct KnowledgeMetadata {
    path: String,
    title: Option<String>,
    source: Option<String>,
    date: Option<String>,
}

/// Execute the list_knowledge tool
pub fn execute_list(args: &Value, project_root: &Path) -> Result<String> {
    let subpath = get_optional_string(args, "path");

    let knowledge_dir = project_root.join("agentd/knowledge");
    if !knowledge_dir.exists() {
        return Ok("No knowledge directory found. Run 'agentd init' first.".to_string());
    }

    let search_dir = match &subpath {
        Some(p) => {
            let normalized = p.trim_start_matches('/').trim_start_matches("./");
            if normalized.contains("..") {
                anyhow::bail!("Invalid path: directory traversal not allowed");
            }
            knowledge_dir.join(normalized)
        }
        None => knowledge_dir.clone(),
    };

    if !search_dir.exists() {
        anyhow::bail!("Directory not found: {}", subpath.unwrap_or_default());
    }

    let mut entries = collect_knowledge_files(&search_dir, &knowledge_dir)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    if entries.is_empty() {
        return Ok("No knowledge files found.".to_string());
    }

    let mut result = String::new();
    if let Some(p) = &subpath {
        result.push_str(&format!("# Knowledge: {}\n\n", p));
    } else {
        result.push_str("# Knowledge Base\n\n");
    }

    for entry in &entries {
        // Format: - **path**: Title (date)
        result.push_str(&format!("- **{}**", entry.path));

        if let Some(title) = &entry.title {
            if let Some(date) = &entry.date {
                result.push_str(&format!(": {} ({})", title, date));
            } else {
                result.push_str(&format!(": {}", title));
            }
        }
        result.push('\n');

        // Add source on next line if available
        if let Some(source) = &entry.source {
            result.push_str(&format!("  Source: {}\n", source));
        }
    }

    result.push_str(&format!("\nTotal: {} file(s)", entries.len()));

    Ok(result)
}

/// Recursively collect knowledge files with their metadata
fn collect_knowledge_files(
    dir: &Path,
    base_dir: &Path,
) -> Result<Vec<KnowledgeMetadata>> {
    let mut entries = Vec::new();

    if !dir.is_dir() {
        return Ok(entries);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectories
            let mut sub_entries = collect_knowledge_files(&path, base_dir)?;
            entries.append(&mut sub_entries);
        } else if path.extension().map_or(false, |ext| ext == "md") {
            let relative_path = path
                .strip_prefix(base_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // Parse frontmatter to get metadata
            let metadata = parse_knowledge_metadata(&path, relative_path);
            entries.push(metadata);
        }
    }

    Ok(entries)
}

/// Parse frontmatter from a knowledge file
fn parse_knowledge_metadata(path: &Path, relative_path: String) -> KnowledgeMetadata {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return KnowledgeMetadata {
                path: relative_path,
                title: None,
                source: None,
                date: None,
            }
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    // Check if file has frontmatter
    if lines.first() != Some(&"---") {
        // No frontmatter, try to get first heading as title
        let title = get_first_heading_from_content(&content);
        return KnowledgeMetadata {
            path: relative_path,
            title,
            source: None,
            date: None,
        };
    }

    // Parse frontmatter
    let mut title = None;
    let mut source = None;
    let mut date = None;

    for line in lines.iter().skip(1) {
        if *line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("title: ") {
            title = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("source: ") {
            source = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("date: ") {
            date = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("updated: ") {
            // Use updated date if available
            date = Some(value.to_string());
        }
    }

    // If no title in frontmatter, try first heading
    if title.is_none() {
        title = get_first_heading_from_content(&content);
    }

    KnowledgeMetadata {
        path: relative_path,
        title,
        source,
        date,
    }
}

/// Extract the first markdown heading from content
fn get_first_heading_from_content(content: &str) -> Option<String> {
    let mut in_frontmatter = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip frontmatter
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            continue;
        }

        if trimmed.starts_with('#') {
            // Remove # symbols and trim
            return Some(trimmed.trim_start_matches('#').trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_knowledge_index() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create knowledge directory
        let knowledge_dir = project_root.join("agentd/knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();
        std::fs::write(
            knowledge_dir.join("index.md"),
            "# Knowledge Base\n\nWelcome.",
        )
        .unwrap();

        let args = json!({
            "path": "index.md"
        });

        let result = execute_read(&args, project_root).unwrap();
        assert!(result.contains("# Knowledge Base"));
        assert!(result.contains("Welcome"));
    }

    #[test]
    fn test_read_knowledge_nested() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create nested knowledge
        let arch_dir = project_root.join("agentd/knowledge/00-architecture");
        std::fs::create_dir_all(&arch_dir).unwrap();
        std::fs::write(
            arch_dir.join("01-overview.md"),
            "# Architecture Overview\n\nThis is the overview.",
        )
        .unwrap();

        let args = json!({
            "path": "00-architecture/01-overview.md"
        });

        let result = execute_read(&args, project_root).unwrap();
        assert!(result.contains("# Architecture Overview"));
    }

    #[test]
    fn test_list_knowledge() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create knowledge structure
        let knowledge_dir = project_root.join("agentd/knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();
        std::fs::write(knowledge_dir.join("index.md"), "# Knowledge Base").unwrap();

        let arch_dir = knowledge_dir.join("00-architecture");
        std::fs::create_dir_all(&arch_dir).unwrap();
        std::fs::write(arch_dir.join("index.md"), "# Architecture").unwrap();
        std::fs::write(arch_dir.join("01-overview.md"), "# Overview").unwrap();

        let args = json!({});

        let result = execute_list(&args, project_root).unwrap();
        assert!(result.contains("index.md"));
        assert!(result.contains("00-architecture/index.md"));
        assert!(result.contains("00-architecture/01-overview.md"));
        assert!(result.contains("Total: 3 file(s)"));
    }

    #[test]
    fn test_list_knowledge_with_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let knowledge_dir = project_root.join("agentd/knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();

        // Create file with frontmatter
        std::fs::write(
            knowledge_dir.join("test.md"),
            "---\ntitle: Test Document\nsource: https://example.com\ndate: 2025-01-19\n---\n\n# Content",
        )
        .unwrap();

        let args = json!({});

        let result = execute_list(&args, project_root).unwrap();
        assert!(result.contains("test.md"));
        assert!(result.contains("Test Document"));
        assert!(result.contains("2025-01-19"));
        assert!(result.contains("https://example.com"));
    }

    #[test]
    fn test_write_knowledge_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "path": "test/new-doc.md",
            "title": "New Document",
            "source": "https://example.com/doc",
            "content": "# Hello\n\nThis is content."
        });

        let result = execute_write(&args, project_root).unwrap();
        assert!(result.contains("✓ Knowledge written: test/new-doc.md"));
        assert!(result.contains("Title: New Document"));
        assert!(result.contains("Source: https://example.com/doc"));

        // Verify file was created with frontmatter
        let file_path = project_root.join("agentd/knowledge/test/new-doc.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("title: New Document"));
        assert!(content.contains("source: https://example.com/doc"));
        assert!(content.contains("date:"));
        assert!(content.contains("# Hello"));
    }

    #[test]
    fn test_write_knowledge_update_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let knowledge_dir = project_root.join("agentd/knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();

        // Create existing file
        std::fs::write(
            knowledge_dir.join("existing.md"),
            "---\ntitle: Old Title\nsource: old-source\ndate: 2024-01-01\n---\n\nOld content",
        )
        .unwrap();

        let args = json!({
            "path": "existing.md",
            "title": "Updated Title",
            "source": "new-source",
            "content": "# Updated\n\nNew content."
        });

        let result = execute_write(&args, project_root).unwrap();
        assert!(result.contains("✓ Knowledge updated: existing.md"));

        // Verify file was updated with original date preserved
        let file_path = project_root.join("agentd/knowledge/existing.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("title: Updated Title"));
        assert!(content.contains("date: 2024-01-01")); // Original date preserved
        assert!(content.contains("updated:")); // Updated date added
        assert!(content.contains("# Updated"));
    }

    #[test]
    fn test_write_knowledge_directory_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "path": "../secrets.md",
            "title": "Evil",
            "source": "evil",
            "content": "evil content"
        });

        let result = execute_write(&args, project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_directory_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let knowledge_dir = project_root.join("agentd/knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();

        let args = json!({
            "path": "../secrets.md"
        });

        let result = execute_read(&args, project_root);
        assert!(result.is_err());
    }
}
