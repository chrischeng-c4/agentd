//! Knowledge MCP Tools
//!
//! Tools for reading and listing knowledge documentation.

use super::{get_optional_string, ToolDefinition};
use crate::Result;
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
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    if entries.is_empty() {
        return Ok("No knowledge files found.".to_string());
    }

    let mut result = String::new();
    if let Some(p) = &subpath {
        result.push_str(&format!("# Knowledge: {}\n\n", p));
    } else {
        result.push_str("# Knowledge Base\n\n");
    }

    for (path, description) in &entries {
        result.push_str(&format!("- **{}**", path));
        if !description.is_empty() {
            result.push_str(&format!(": {}", description));
        }
        result.push('\n');
    }

    result.push_str(&format!("\nTotal: {} file(s)", entries.len()));

    Ok(result)
}

/// Recursively collect knowledge files with their first heading as description
fn collect_knowledge_files(
    dir: &Path,
    base_dir: &Path,
) -> Result<Vec<(String, String)>> {
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

            // Get first heading as description
            let description = get_first_heading(&path).unwrap_or_default();

            entries.push((relative_path, description));
        }
    }

    Ok(entries)
}

/// Extract the first markdown heading from a file
fn get_first_heading(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
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
