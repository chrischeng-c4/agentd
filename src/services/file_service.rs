//! File service - Business logic for reading change files

use crate::Result;
use std::path::Path;

/// Read a file from a change directory (proposal.md, specs/*.md, tasks.md)
pub fn read_file(change_id: &str, file: &str, project_root: &Path) -> Result<String> {
    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    // Determine which file to read
    let file_path = match file {
        "proposal" => change_dir.join("proposal.md"),
        "tasks" => change_dir.join("tasks.md"),
        spec_name => {
            // Try as a spec file
            let spec_path = change_dir.join("specs").join(format!("{}.md", spec_name));
            if spec_path.exists() {
                spec_path
            } else {
                // Maybe they included .md already
                let spec_path_with_ext = change_dir.join("specs").join(spec_name);
                if spec_path_with_ext.exists() {
                    spec_path_with_ext
                } else {
                    anyhow::bail!(
                        "File not found: '{}'. Use 'proposal', 'tasks', or a spec name.",
                        file
                    );
                }
            }
        }
    };

    if !file_path.exists() {
        anyhow::bail!("File not found: {}", file_path.display());
    }

    let content = std::fs::read_to_string(&file_path)?;

    Ok(format!(
        "# File: {}\n\n{}",
        file_path
            .strip_prefix(&change_dir)
            .unwrap_or(&file_path)
            .display(),
        content
    ))
}

/// List all spec files in a change directory
pub fn list_specs(change_id: &str, project_root: &Path) -> Result<String> {
    // Check change directory exists
    let change_dir = project_root.join("agentd/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found.", change_id);
    }

    let specs_dir = change_dir.join("specs");
    if !specs_dir.exists() {
        return Ok("No specs directory found.".to_string());
    }

    let mut specs = Vec::new();
    for entry in std::fs::read_dir(&specs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            if let Some(name) = path.file_stem() {
                let name_str = name.to_string_lossy();
                // Skip skeleton files
                if !name_str.starts_with('_') {
                    specs.push(name_str.to_string());
                }
            }
        }
    }

    if specs.is_empty() {
        return Ok("No spec files found.".to_string());
    }

    specs.sort();

    let mut result = format!("# Specs for change '{}'\n\n", change_id);
    for spec in &specs {
        result.push_str(&format!("- {}\n", spec));
    }
    result.push_str(&format!("\nTotal: {} spec(s)", specs.len()));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with proposal
        let change_dir = project_root.join("agentd/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nThis is a test.",
        )
        .unwrap();

        let result = read_file("test-change", "proposal", project_root).unwrap();
        assert!(result.contains("# Test Proposal"));
        assert!(result.contains("This is a test"));
    }

    #[test]
    fn test_read_spec() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with spec
        let specs_dir = project_root.join("agentd/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("my-feature.md"),
            "# My Feature Spec\n\nRequirements here.",
        )
        .unwrap();

        let result = read_file("test-change", "my-feature", project_root).unwrap();
        assert!(result.contains("# My Feature Spec"));
    }

    #[test]
    fn test_list_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory with specs
        let specs_dir = project_root.join("agentd/changes/test-change/specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(specs_dir.join("spec-a.md"), "# Spec A").unwrap();
        std::fs::write(specs_dir.join("spec-b.md"), "# Spec B").unwrap();
        std::fs::write(specs_dir.join("_skeleton.md"), "# Skeleton").unwrap();

        let result = list_specs("test-change", project_root).unwrap();
        assert!(result.contains("spec-a"));
        assert!(result.contains("spec-b"));
        assert!(!result.contains("_skeleton"));
        assert!(result.contains("Total: 2 spec(s)"));
    }
}
