//! Integration tests for the plan viewer
//!
//! These tests verify HTML rendering, annotation functionality,
//! and path security measures.

#[cfg(feature = "ui")]
mod ui_tests {
    use agentd::models::{Annotation, AnnotationStore, get_author_name};
    use agentd::ui::viewer::{slugify, ViewerManager};
    use std::fs;
    use tempfile::TempDir;

    // =========================================================================
    // Slugify Tests
    // =========================================================================

    #[test]
    fn test_slugify_requirement_heading() {
        assert_eq!(
            slugify("R1: Native Window Rendering"),
            "r1-native-window-rendering"
        );
    }

    #[test]
    fn test_slugify_scenario_heading() {
        assert_eq!(
            slugify("Scenario: Open viewer for valid change"),
            "scenario-open-viewer-for-valid-change"
        );
    }

    #[test]
    fn test_slugify_preserves_numbers() {
        assert_eq!(slugify("Task 1.1: Create models"), "task-1-1-create-models");
    }

    #[test]
    fn test_slugify_handles_special_characters() {
        assert_eq!(slugify("Hello (World)!"), "hello-world");
        assert_eq!(slugify("Test: [Something]"), "test-something");
        assert_eq!(slugify("A & B"), "a-b");
    }

    #[test]
    fn test_slugify_handles_unicode() {
        // Unicode letters like é are alphanumeric and kept
        assert_eq!(slugify("Cafe Résumé"), "cafe-résumé");
    }

    #[test]
    fn test_slugify_empty_string() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_slugify_only_special_chars() {
        assert_eq!(slugify("!!!"), "");
        assert_eq!(slugify("---"), "");
    }

    // =========================================================================
    // Annotation Tests
    // =========================================================================

    #[test]
    fn test_annotation_creation_with_metadata() {
        let annotation = Annotation::new(
            "proposal.md",
            "r1-native-window",
            "Use tao 0.30+ for better macOS support",
            "test-user",
        );

        assert_eq!(annotation.file, "proposal.md");
        assert_eq!(annotation.section_id, "r1-native-window");
        assert_eq!(annotation.content, "Use tao 0.30+ for better macOS support");
        assert_eq!(annotation.author, "test-user");
        assert!(!annotation.resolved);

        // Verify UUID is valid
        assert!(uuid::Uuid::parse_str(&annotation.id).is_ok());

        // Verify timestamp is valid ISO 8601
        assert!(chrono::DateTime::parse_from_rfc3339(&annotation.created_at).is_ok());
    }

    #[test]
    fn test_annotation_store_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        // Create and save
        let mut store = AnnotationStore::new("test-change");
        store.add(Annotation::new("proposal.md", "r1", "Comment 1", "user1"));
        store.add(Annotation::new("proposal.md", "r2", "Comment 2", "user2"));
        store.add(Annotation::new("CHALLENGE.md", "issue1", "Challenge comment", "user1"));

        store.save(&path).unwrap();

        // Load and verify
        let loaded = AnnotationStore::load(&path, "test-change").unwrap();
        assert_eq!(loaded.change_id, "test-change");
        assert_eq!(loaded.len(), 3);

        // Verify filtering
        let proposal_annotations = loaded.for_file("proposal.md");
        assert_eq!(proposal_annotations.len(), 2);

        let r1_annotations = loaded.for_section("proposal.md", "r1");
        assert_eq!(r1_annotations.len(), 1);
        assert_eq!(r1_annotations[0].content, "Comment 1");
    }

    #[test]
    fn test_annotation_store_resilience_malformed_json() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        // Write malformed JSON
        fs::write(&path, "{ not valid json }").unwrap();

        // Should not panic, should return empty store
        let store = AnnotationStore::load(&path, "test-change").unwrap();
        assert!(store.is_empty());

        // Backup should exist
        let backup_path = path.with_extension("json.bak");
        assert!(backup_path.exists());
    }

    #[test]
    fn test_annotation_store_resilience_partial_json() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        // Write partial JSON (missing closing brace)
        fs::write(&path, r#"{"change_id": "test", "annotations": ["#).unwrap();

        // Should not panic
        let store = AnnotationStore::load(&path, "test-change").unwrap();
        assert!(store.is_empty());
    }

    #[test]
    fn test_annotation_resolve() {
        let mut store = AnnotationStore::new("test-change");
        let annotation = Annotation::new("proposal.md", "r1", "Comment", "user");
        let id = annotation.id.clone();
        store.add(annotation);

        assert_eq!(store.unresolved_count(), 1);

        store.resolve(&id).unwrap();

        assert_eq!(store.unresolved_count(), 0);
        assert!(store.find(&id).unwrap().resolved);
    }

    #[test]
    fn test_get_author_name_returns_something() {
        let author = get_author_name();
        assert!(!author.is_empty());
    }

    // =========================================================================
    // ViewerManager Tests
    // =========================================================================

    fn setup_change(temp_dir: &TempDir, change_id: &str) -> std::path::PathBuf {
        let change_dir = temp_dir.path().join("agentd/changes").join(change_id);
        fs::create_dir_all(&change_dir).unwrap();
        change_dir
    }

    #[test]
    fn test_viewer_manager_change_exists() {
        let temp_dir = TempDir::new().unwrap();
        setup_change(&temp_dir, "my-change");

        let manager = ViewerManager::new("my-change", temp_dir.path());
        assert!(manager.change_exists());

        let manager2 = ViewerManager::new("nonexistent", temp_dir.path());
        assert!(!manager2.change_exists());
    }

    #[test]
    fn test_viewer_manager_load_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_change(&temp_dir, "test-change");

        fs::write(
            change_dir.join("proposal.md"),
            "# My Proposal\n\nThis is the content.\n\n## Summary\n\nDetails here.",
        )
        .unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert!(response.exists);
        assert!(response.content.contains("My Proposal"));
        assert!(response.content.contains("<h1"));
        assert!(response.content.contains("id=\"my-proposal\""));
    }

    #[test]
    fn test_viewer_manager_load_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        setup_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("CHALLENGE.md").unwrap();

        assert!(!response.exists);
        assert!(response.content.contains("File not found"));
    }

    #[test]
    fn test_viewer_manager_load_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_change(&temp_dir, "test-change");

        fs::write(
            change_dir.join("STATE.yaml"),
            "phase: Challenged\nchange_id: test-change\n",
        )
        .unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("STATE.yaml").unwrap();

        assert!(response.exists);
        assert!(response.content.contains("language-yaml"));
        assert!(response.content.contains("phase: Challenged"));
    }

    // =========================================================================
    // Path Traversal Security Tests
    // =========================================================================

    #[test]
    fn test_path_traversal_dotdot_rejected() {
        let temp_dir = TempDir::new().unwrap();
        setup_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        let result = manager.load_file("../secret.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_slash_rejected() {
        let temp_dir = TempDir::new().unwrap();
        setup_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        let result = manager.load_file("foo/bar.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_backslash_rejected() {
        let temp_dir = TempDir::new().unwrap();
        setup_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        let result = manager.load_file("foo\\bar.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_not_in_allowlist_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_change(&temp_dir, "test-change");

        // Create a file that's not in the allowlist
        fs::write(change_dir.join("secret.txt"), "secret content").unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let result = manager.load_file("secret.txt");

        assert!(result.is_err());
    }

    #[test]
    fn test_allowed_files_can_be_loaded() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_change(&temp_dir, "test-change");

        // Create all allowed files
        fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
        fs::write(change_dir.join("CHALLENGE.md"), "# Challenge").unwrap();
        fs::write(change_dir.join("STATE.yaml"), "phase: test").unwrap();
        fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());

        // All should load successfully
        assert!(manager.load_file("proposal.md").is_ok());
        assert!(manager.load_file("CHALLENGE.md").is_ok());
        assert!(manager.load_file("STATE.yaml").is_ok());
        assert!(manager.load_file("tasks.md").is_ok());
    }

    // =========================================================================
    // HTML Rendering Tests
    // =========================================================================

    #[test]
    fn test_html_rendering_includes_heading_ids() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_change(&temp_dir, "test-change");

        let markdown = r#"# Overview

This is the overview.

## Requirements

### R1: First Requirement

Details about R1.

### R2: Second Requirement

Details about R2.
"#;

        fs::write(change_dir.join("proposal.md"), markdown).unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert!(response.content.contains("id=\"overview\""));
        assert!(response.content.contains("id=\"requirements\""));
        assert!(response.content.contains("id=\"r1-first-requirement\""));
        assert!(response.content.contains("id=\"r2-second-requirement\""));
    }

    #[test]
    fn test_html_rendering_handles_code_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_change(&temp_dir, "test-change");

        let markdown = r#"# Code Example

```rust
fn main() {
    println!("Hello");
}
```
"#;

        fs::write(change_dir.join("proposal.md"), markdown).unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert!(response.content.contains("<code"));
        assert!(response.content.contains("language-rust"));
    }

    #[test]
    fn test_html_rendering_handles_tables() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_change(&temp_dir, "test-change");

        let markdown = r#"# Table Example

| Column A | Column B |
|----------|----------|
| Value 1  | Value 2  |
"#;

        fs::write(change_dir.join("proposal.md"), markdown).unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert!(response.content.contains("<table>"));
        assert!(response.content.contains("<th>"));
        assert!(response.content.contains("<td>"));
    }
}

// Tests that run without the ui feature
mod common_tests {
    use agentd::models::{Annotation, AnnotationStore};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_annotation_serialization_roundtrip() {
        let annotation = Annotation::new(
            "proposal.md",
            "r1-test",
            "Test comment",
            "test-user",
        );

        let json = serde_json::to_string(&annotation).unwrap();
        let parsed: Annotation = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, annotation.id);
        assert_eq!(parsed.file, annotation.file);
        assert_eq!(parsed.section_id, annotation.section_id);
        assert_eq!(parsed.content, annotation.content);
        assert_eq!(parsed.author, annotation.author);
        assert_eq!(parsed.created_at, annotation.created_at);
        assert_eq!(parsed.resolved, annotation.resolved);
    }

    #[test]
    fn test_annotation_store_json_format() {
        let mut store = AnnotationStore::new("my-change");
        store.add(Annotation::new("proposal.md", "r1", "Comment", "user"));

        let json = serde_json::to_string_pretty(&store).unwrap();

        // Verify the JSON structure matches the spec
        assert!(json.contains("\"change_id\""));
        assert!(json.contains("\"annotations\""));
        assert!(json.contains("\"id\""));
        assert!(json.contains("\"file\""));
        assert!(json.contains("\"section_id\""));
        assert!(json.contains("\"content\""));
        assert!(json.contains("\"author\""));
        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"resolved\""));
    }

    #[test]
    fn test_annotation_store_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        let mut store = AnnotationStore::new("test");
        store.add(Annotation::new("file.md", "section", "content", "author"));

        // Save should create the file
        store.save(&path).unwrap();
        assert!(path.exists());

        // Content should be valid JSON
        let content = fs::read_to_string(&path).unwrap();
        let _: AnnotationStore = serde_json::from_str(&content).unwrap();

        // Temp file should not exist after save
        let temp_path = path.with_extension("json.tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_annotation_store_nonexistent_file_returns_empty() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let store = AnnotationStore::load(&path, "test").unwrap();
        assert!(store.is_empty());
        assert_eq!(store.change_id, "test");
    }
}
