//! Viewer state and file loading management
//!
//! Handles loading plan files and annotations from the file system,
//! with path validation to prevent directory traversal attacks.

use crate::models::{AnnotationStore, AnnotationResult, AnnotationError};
use super::render::{render_markdown_to_html, render_yaml_to_html, render_not_found_html};
use std::fs;
use std::path::{Path, PathBuf};

/// Base allowed files that can be loaded in the viewer
const BASE_ALLOWED_FILES: &[&str] = &["proposal.md", "CHALLENGE.md", "STATE.yaml", "tasks.md"];

/// Response from loading a file, includes content and annotations
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileLoadResponse {
    /// Rendered HTML content
    pub content: String,
    /// Annotations for this file
    pub annotations: Vec<crate::models::Annotation>,
    /// Whether the file exists
    pub exists: bool,
}

/// Viewer manager for handling file operations
pub struct ViewerManager {
    /// Change ID being viewed
    change_id: String,
    /// Path to the change directory
    change_dir: PathBuf,
    /// Path to annotations file
    annotations_path: PathBuf,
}

impl ViewerManager {
    /// Create a new viewer manager for a change
    pub fn new(change_id: &str, project_root: &Path) -> Self {
        let change_dir = project_root.join("agentd/changes").join(change_id);
        let annotations_path = change_dir.join("annotations.json");

        Self {
            change_id: change_id.to_string(),
            change_dir,
            annotations_path,
        }
    }

    /// Check if the change directory exists
    pub fn change_exists(&self) -> bool {
        self.change_dir.exists() && self.change_dir.is_dir()
    }

    /// Get the change directory path
    pub fn change_dir(&self) -> &Path {
        &self.change_dir
    }

    /// Load a file and its annotations
    ///
    /// Returns rendered HTML content and associated annotations.
    /// Returns a "not found" placeholder if the file doesn't exist.
    pub fn load_file(&self, filename: &str) -> Result<FileLoadResponse, ViewerError> {
        // Validate filename to prevent path traversal
        self.validate_filename(filename)?;

        let file_path = self.change_dir.join(filename);
        let annotations = self.load_annotations()?;

        if !file_path.exists() {
            return Ok(FileLoadResponse {
                content: render_not_found_html(filename),
                annotations: annotations.for_file(filename).into_iter().cloned().collect(),
                exists: false,
            });
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| ViewerError::FileRead(filename.to_string(), e))?;

        let html = if filename.ends_with(".yaml") || filename.ends_with(".yml") {
            render_yaml_to_html(&content)
        } else {
            render_markdown_to_html(&content)
        };

        Ok(FileLoadResponse {
            content: html,
            annotations: annotations.for_file(filename).into_iter().cloned().collect(),
            exists: true,
        })
    }

    /// Load annotations from the annotations file
    pub fn load_annotations(&self) -> AnnotationResult<AnnotationStore> {
        AnnotationStore::load(&self.annotations_path, &self.change_id)
    }

    /// Save annotations to the annotations file
    pub fn save_annotations(&self, store: &AnnotationStore) -> AnnotationResult<()> {
        store.save(&self.annotations_path)
    }

    /// Get the annotations file path
    pub fn annotations_path(&self) -> &Path {
        &self.annotations_path
    }

    /// Validate that a filename is allowed and doesn't contain path traversal
    fn validate_filename(&self, filename: &str) -> Result<(), ViewerError> {
        // Check for path traversal attempts (but allow specs/ prefix)
        if filename.contains("..") || filename.contains('\\') {
            return Err(ViewerError::PathTraversal(filename.to_string()));
        }

        // Check if it's a base file or specs file
        let is_base_file = BASE_ALLOWED_FILES.contains(&filename);
        let is_specs_file = filename.starts_with("specs/")
            && filename.ends_with(".md")
            && !filename.contains("..");

        if !is_base_file && !is_specs_file {
            return Err(ViewerError::FileNotAllowed(filename.to_string()));
        }

        // Double-check the resolved path is within change_dir
        let resolved = self.change_dir.join(filename);
        if !resolved.starts_with(&self.change_dir) {
            return Err(ViewerError::PathTraversal(filename.to_string()));
        }

        Ok(())
    }

    /// List all available files in the change directory
    pub fn list_files(&self) -> Vec<FileInfo> {
        let mut files: Vec<FileInfo> = BASE_ALLOWED_FILES
            .iter()
            .map(|&name| {
                let path = self.change_dir.join(name);
                FileInfo {
                    name: name.to_string(),
                    exists: path.exists(),
                }
            })
            .collect();

        // Add specs/*.md files
        let specs_dir = self.change_dir.join("specs");
        if specs_dir.exists() && specs_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&specs_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "md") {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            // Skip skeleton files
                            if !name.starts_with('_') {
                                files.push(FileInfo {
                                    name: format!("specs/{}", name),
                                    exists: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        files
    }

    /// Update the phase in STATE.yaml
    pub fn update_phase(&self, new_phase: &str) -> Result<(), ViewerError> {
        let state_path = self.change_dir.join("STATE.yaml");

        if !state_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "STATE.yaml not found"
            ).into());
        }

        let content = fs::read_to_string(&state_path)?;

        // Simple regex-like replacement for phase field
        let updated = content
            .lines()
            .map(|line| {
                if line.starts_with("phase:") {
                    format!("phase: {}", new_phase)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Atomic write
        let temp_path = state_path.with_extension("yaml.tmp");
        fs::write(&temp_path, &updated)?;
        fs::rename(&temp_path, &state_path)?;

        Ok(())
    }
}

/// Information about a file in the change directory
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    /// Filename
    pub name: String,
    /// Whether the file exists
    pub exists: bool,
}

/// Errors that can occur in the viewer manager
#[derive(Debug, thiserror::Error)]
pub enum ViewerError {
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),

    #[error("File not allowed: {0}")]
    FileNotAllowed(String),

    #[error("Failed to read file '{0}': {1}")]
    FileRead(String, std::io::Error),

    #[error("Annotation error: {0}")]
    Annotation(#[from] AnnotationError),

    #[error("Change not found: {0}")]
    ChangeNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_change(dir: &TempDir, change_id: &str) -> PathBuf {
        let change_dir = dir.path().join("agentd/changes").join(change_id);
        fs::create_dir_all(&change_dir).unwrap();
        change_dir
    }

    #[test]
    fn test_viewer_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ViewerManager::new("my-change", temp_dir.path());

        assert_eq!(manager.change_id, "my-change");
        assert!(manager.change_dir.ends_with("agentd/changes/my-change"));
    }

    #[test]
    fn test_change_exists() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "existing-change");

        let manager = ViewerManager::new("existing-change", temp_dir.path());
        assert!(manager.change_exists());

        let manager2 = ViewerManager::new("nonexistent", temp_dir.path());
        assert!(!manager2.change_exists());
    }

    #[test]
    fn test_load_file_existing() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(
            change_dir.join("proposal.md"),
            "# My Proposal\n\nThis is content.",
        )
        .unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert!(response.exists);
        assert!(response.content.contains("<h1"));
        assert!(response.content.contains("My Proposal"));
    }

    #[test]
    fn test_load_file_missing() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("CHALLENGE.md").unwrap();

        assert!(!response.exists);
        assert!(response.content.contains("File not found"));
        assert!(response.content.contains("CHALLENGE.md"));
    }

    #[test]
    fn test_load_yaml_file() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(
            change_dir.join("STATE.yaml"),
            "phase: Proposed\nchange_id: test",
        )
        .unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("STATE.yaml").unwrap();

        assert!(response.exists);
        assert!(response.content.contains("language-yaml"));
        assert!(response.content.contains("phase: Proposed"));
    }

    #[test]
    fn test_path_traversal_rejected() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        // Test various path traversal attempts
        assert!(matches!(
            manager.load_file("../secret.txt"),
            Err(ViewerError::PathTraversal(_))
        ));
        assert!(matches!(
            manager.load_file("..\\secret.txt"),
            Err(ViewerError::PathTraversal(_))
        ));
        // foo/bar.txt is not a path traversal, but it's not allowed (not specs/*.md)
        assert!(matches!(
            manager.load_file("foo/bar.txt"),
            Err(ViewerError::FileNotAllowed(_))
        ));
    }

    #[test]
    fn test_file_not_allowed() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        assert!(matches!(
            manager.load_file("secret.txt"),
            Err(ViewerError::FileNotAllowed(_))
        ));
        assert!(matches!(
            manager.load_file("config.toml"),
            Err(ViewerError::FileNotAllowed(_))
        ));
    }

    #[test]
    fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(change_dir.join("proposal.md"), "content").unwrap();
        fs::write(change_dir.join("tasks.md"), "content").unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let files = manager.list_files();

        assert_eq!(files.len(), 4); // All BASE_ALLOWED_FILES (no specs dir)

        let proposal = files.iter().find(|f| f.name == "proposal.md").unwrap();
        assert!(proposal.exists);

        let challenge = files.iter().find(|f| f.name == "CHALLENGE.md").unwrap();
        assert!(!challenge.exists);
    }

    #[test]
    fn test_load_file_with_annotations() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = setup_test_change(&temp_dir, "test-change");

        fs::write(change_dir.join("proposal.md"), "# Heading").unwrap();

        // Create annotations
        let mut store = AnnotationStore::new("test-change");
        store.add(crate::models::Annotation::new(
            "proposal.md",
            "heading",
            "comment",
            "user",
        ));
        store.add(crate::models::Annotation::new(
            "CHALLENGE.md",
            "issue",
            "other",
            "user",
        ));

        let annotations_path = change_dir.join("annotations.json");
        store.save(&annotations_path).unwrap();

        let manager = ViewerManager::new("test-change", temp_dir.path());
        let response = manager.load_file("proposal.md").unwrap();

        assert_eq!(response.annotations.len(), 1);
        assert_eq!(response.annotations[0].file, "proposal.md");
    }

    #[test]
    fn test_validate_filename_empty() {
        let temp_dir = TempDir::new().unwrap();
        setup_test_change(&temp_dir, "test-change");

        let manager = ViewerManager::new("test-change", temp_dir.path());

        // Empty string should fail
        assert!(manager.validate_filename("").is_err());
    }
}
