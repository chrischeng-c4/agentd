//! Knowledge service - Business logic for knowledge base operations

use crate::Result;
use std::path::Path;

/// Knowledge file metadata
#[derive(Debug, Clone)]
pub struct KnowledgeMetadata {
    pub path: String,
    pub title: Option<String>,
    pub source: Option<String>,
    pub date: Option<String>,
}

/// Read a knowledge document
pub fn read_knowledge(path: &str, project_root: &Path) -> Result<String> {
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

/// List all knowledge documents with their metadata
pub fn list_knowledge(subpath: Option<&str>, project_root: &Path) -> Result<String> {
    let knowledge_dir = project_root.join("agentd/knowledge");
    if !knowledge_dir.exists() {
        return Ok("No knowledge directory found. Run 'agentd init' first.".to_string());
    }

    let search_dir = match subpath {
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
    if let Some(p) = subpath {
        result.push_str(&format!("# Knowledge: {}\n\n", p));
    } else {
        result.push_str("# Knowledge Base\n\n");
    }

    for entry in &entries {
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
fn collect_knowledge_files(dir: &Path, base_dir: &Path) -> Result<Vec<KnowledgeMetadata>> {
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

/// Input for writing knowledge documents
#[derive(Debug)]
pub struct WriteKnowledgeInput {
    pub path: String,
    pub title: String,
    pub source: String,
    pub content: String,
}

/// Write or update a knowledge document with auto-generated frontmatter
pub fn write_knowledge(input: WriteKnowledgeInput, project_root: &Path) -> Result<String> {
    use chrono::Local;

    let knowledge_dir = project_root.join("agentd/knowledge");
    if !knowledge_dir.exists() {
        std::fs::create_dir_all(&knowledge_dir)?;
    }

    // Normalize path and prevent directory traversal
    let normalized_path = input.path.trim_start_matches('/').trim_start_matches("./");
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
            input.title, input.source, original_date, today
        )
    } else {
        format!(
            "---\ntitle: {}\nsource: {}\ndate: {}\n---",
            input.title, input.source, today
        )
    };

    // Combine frontmatter and content
    let full_content = format!("{}\n\n{}", frontmatter, input.content.trim());

    std::fs::write(&file_path, full_content)?;

    let action = if is_update { "updated" } else { "written" };
    let mut result = format!("✓ Knowledge {}: {}\n", action, normalized_path);
    result.push_str(&format!("  Title: {}\n", input.title));
    result.push_str(&format!("  Source: {}\n", input.source));
    result.push_str(&format!("  Date: {}", today));

    Ok(result)
}

/// Write or update a spec in the main agentd/specs/ directory (for archive merge)
pub fn write_main_spec(path: &str, content: &str, project_root: &Path) -> Result<String> {
    let specs_dir = project_root.join("agentd/specs");
    if !specs_dir.exists() {
        std::fs::create_dir_all(&specs_dir)?;
    }

    // Normalize path and prevent directory traversal
    let normalized_path = path.trim_start_matches('/').trim_start_matches("./");
    if normalized_path.contains("..") {
        anyhow::bail!("Invalid path: directory traversal not allowed");
    }

    let file_path = specs_dir.join(normalized_path);

    // Security: ensure path is within specs directory
    if !file_path.starts_with(&specs_dir) {
        anyhow::bail!("Invalid path: must be within agentd/specs/");
    }

    // Create parent directories if needed
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let is_update = file_path.exists();
    std::fs::write(&file_path, content)?;

    let action = if is_update { "updated" } else { "created" };
    Ok(format!("✓ Spec {}: agentd/specs/{}", action, normalized_path))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_knowledge() {
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

        let result = read_knowledge("index.md", project_root).unwrap();
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

        let result = read_knowledge("00-architecture/01-overview.md", project_root).unwrap();
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

        let result = list_knowledge(None, project_root).unwrap();
        assert!(result.contains("index.md"));
        assert!(result.contains("00-architecture/index.md"));
        assert!(result.contains("00-architecture/01-overview.md"));
        assert!(result.contains("Total: 3 file(s)"));
    }

    #[test]
    fn test_directory_traversal_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let knowledge_dir = project_root.join("agentd/knowledge");
        std::fs::create_dir_all(&knowledge_dir).unwrap();

        let result = read_knowledge("../secrets.md", project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_knowledge_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let input = WriteKnowledgeInput {
            path: "test/new-doc.md".to_string(),
            title: "New Document".to_string(),
            source: "https://example.com/doc".to_string(),
            content: "# Hello\n\nThis is content.".to_string(),
        };

        let result = write_knowledge(input, project_root).unwrap();
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

        let input = WriteKnowledgeInput {
            path: "existing.md".to_string(),
            title: "Updated Title".to_string(),
            source: "new-source".to_string(),
            content: "# Updated\n\nNew content.".to_string(),
        };

        let result = write_knowledge(input, project_root).unwrap();
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
    fn test_write_main_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let content = "---\ntitle: Test Spec\n---\n\n# Test Spec\n\nContent here.";
        let result = write_main_spec("test-spec.md", content, project_root).unwrap();
        assert!(result.contains("✓ Spec created: agentd/specs/test-spec.md"));

        let file_path = project_root.join("agentd/specs/test-spec.md");
        assert!(file_path.exists());
        let file_content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(file_content, content);
    }
}
