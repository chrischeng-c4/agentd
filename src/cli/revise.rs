//! Revise command implementation
//!
//! Reads annotations from the plan viewer and outputs them formatted
//! for an agent to address. Can optionally trigger reproposal.

use crate::models::AnnotationStore;
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::path::Path;

/// Run the revise command
///
/// Reads annotations from the change directory and outputs them formatted
/// for an agent to understand and address.
pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;
    let change_dir = project_root.join("agentd/changes").join(change_id);

    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", change_id);
    }

    let annotations_path = change_dir.join("annotations.json");
    let store = AnnotationStore::load(&annotations_path, change_id)?;

    if store.is_empty() {
        println!("{}", "No annotations found for this change.".yellow());
        return Ok(());
    }

    let unresolved = store.unresolved_count();
    let total = store.len();

    println!(
        "{}",
        format!(
            "📝 {} of {} annotation(s) need attention:\n",
            unresolved, total
        )
        .cyan()
    );

    // Group annotations by file
    let mut by_file: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
    for annotation in &store.annotations {
        if !annotation.resolved {
            by_file
                .entry(&annotation.file)
                .or_default()
                .push(annotation);
        }
    }

    // Output formatted for agent consumption
    for (file, annotations) in by_file {
        println!("{}", format!("## {}", file).bold());
        println!();

        for annotation in annotations {
            println!(
                "{} (section: {})",
                "•".cyan(),
                annotation.section_id.dimmed()
            );
            println!("  {}", annotation.content);
            println!(
                "  {} by {} at {}",
                "—".dimmed(),
                annotation.author.dimmed(),
                annotation.created_at.dimmed()
            );
            println!();
        }
    }

    // Output actionable instructions
    println!("{}", "─".repeat(60).dimmed());
    println!(
        "{}",
        "Please address these comments and update the proposal accordingly.".yellow()
    );
    println!(
        "{}",
        format!(
            "After making changes, run: agentd proposal {} \"<description of changes>\"",
            change_id
        )
        .dimmed()
    );

    Ok(())
}

/// Get unresolved annotations as a formatted string for agent context
pub fn get_annotations_context(change_dir: &Path, change_id: &str) -> Result<Option<String>> {
    let annotations_path = change_dir.join("annotations.json");
    let store = AnnotationStore::load(&annotations_path, change_id)?;

    if store.is_empty() || store.unresolved_count() == 0 {
        return Ok(None);
    }

    let mut output = String::new();
    output.push_str("# Review Comments to Address\n\n");

    // Group annotations by file
    let mut by_file: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
    for annotation in &store.annotations {
        if !annotation.resolved {
            by_file
                .entry(&annotation.file)
                .or_default()
                .push(annotation);
        }
    }

    for (file, annotations) in by_file {
        output.push_str(&format!("## {}\n\n", file));

        for annotation in annotations {
            output.push_str(&format!(
                "- **Section `{}`**: {}\n",
                annotation.section_id, annotation.content
            ));
        }
        output.push('\n');
    }

    Ok(Some(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Annotation;
    use tempfile::TempDir;

    #[test]
    fn test_get_annotations_context_empty() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();

        let result = get_annotations_context(change_dir, "test-change").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_annotations_context_with_annotations() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();

        // Create annotations
        let mut store = AnnotationStore::new("test-change");
        store.add(Annotation::new(
            "proposal.md",
            "r1-feature",
            "Consider adding error handling",
            "reviewer",
        ));
        store.add(Annotation::new(
            "proposal.md",
            "r2-api",
            "API design needs revision",
            "reviewer",
        ));

        let annotations_path = change_dir.join("annotations.json");
        store.save(&annotations_path).unwrap();

        let result = get_annotations_context(change_dir, "test-change").unwrap();
        assert!(result.is_some());

        let context = result.unwrap();
        assert!(context.contains("Review Comments to Address"));
        assert!(context.contains("proposal.md"));
        assert!(context.contains("r1-feature"));
        assert!(context.contains("Consider adding error handling"));
        assert!(context.contains("r2-api"));
        assert!(context.contains("API design needs revision"));
    }

    #[test]
    fn test_get_annotations_context_all_resolved() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path();

        // Create resolved annotation
        let mut store = AnnotationStore::new("test-change");
        let mut annotation = Annotation::new(
            "proposal.md",
            "r1-feature",
            "Consider adding error handling",
            "reviewer",
        );
        annotation.resolve();
        store.add(annotation);

        let annotations_path = change_dir.join("annotations.json");
        store.save(&annotations_path).unwrap();

        let result = get_annotations_context(change_dir, "test-change").unwrap();
        assert!(result.is_none());
    }
}
