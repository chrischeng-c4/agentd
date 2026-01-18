//! Annotation data models for plan viewer
//!
//! Annotations allow human reviewers to add comments and feedback to specific
//! sections of an agentd plan. These annotations are persisted and can be used
//! to guide subsequent AI-driven refinement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// A single annotation comment on a plan section
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Annotation {
    /// Unique identifier (UUID v4)
    pub id: String,

    /// File being annotated (e.g., "proposal.md")
    pub file: String,

    /// Section ID (slugified heading text, e.g., "r1-native-window-rendering")
    pub section_id: String,

    /// Annotation content/comment
    pub content: String,

    /// Author name (auto-populated from git config or OS user)
    pub author: String,

    /// Creation timestamp (ISO 8601 format)
    pub created_at: String,

    /// Whether the annotation has been resolved
    #[serde(default)]
    pub resolved: bool,
}

impl Annotation {
    /// Create a new annotation with auto-generated ID and timestamp
    pub fn new(
        file: impl Into<String>,
        section_id: impl Into<String>,
        content: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file: file.into(),
            section_id: section_id.into(),
            content: content.into(),
            author: author.into(),
            created_at: Utc::now().to_rfc3339(),
            resolved: false,
        }
    }

    /// Create an annotation with a specific timestamp (for testing)
    pub fn with_timestamp(
        file: impl Into<String>,
        section_id: impl Into<String>,
        content: impl Into<String>,
        author: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file: file.into(),
            section_id: section_id.into(),
            content: content.into(),
            author: author.into(),
            created_at: created_at.to_rfc3339(),
            resolved: false,
        }
    }

    /// Mark the annotation as resolved
    pub fn resolve(&mut self) {
        self.resolved = true;
    }
}

/// Result type for annotation store operations
pub type AnnotationResult<T> = Result<T, AnnotationError>;

/// Errors that can occur when working with annotations
#[derive(Debug, thiserror::Error)]
pub enum AnnotationError {
    #[error("Failed to read annotations file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse annotations JSON: {0}")]
    ParseError(String),

    #[error("Failed to serialize annotations: {0}")]
    SerializeError(#[from] serde_json::Error),

    #[error("Annotation not found: {0}")]
    NotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Container for all annotations associated with a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationStore {
    /// Change ID this store belongs to
    pub change_id: String,

    /// List of all annotations
    pub annotations: Vec<Annotation>,
}

impl AnnotationStore {
    /// Create a new empty annotation store
    pub fn new(change_id: impl Into<String>) -> Self {
        Self {
            change_id: change_id.into(),
            annotations: Vec::new(),
        }
    }

    /// Load annotations from a file, handling corruption gracefully
    ///
    /// If the file doesn't exist, returns an empty store.
    /// If the file is malformed, backs it up and returns an empty store.
    pub fn load(path: &Path, change_id: &str) -> AnnotationResult<Self> {
        if !path.exists() {
            return Ok(Self::new(change_id));
        }

        let content = fs::read_to_string(path)?;

        match serde_json::from_str::<AnnotationStore>(&content) {
            Ok(store) => Ok(store),
            Err(e) => {
                // Log warning and backup corrupt file
                eprintln!(
                    "Warning: Failed to parse annotations.json: {}. Backing up and starting fresh.",
                    e
                );

                // Create backup
                let backup_path = path.with_extension("json.bak");
                if let Err(backup_err) = fs::rename(path, &backup_path) {
                    eprintln!("Warning: Failed to backup corrupt file: {}", backup_err);
                }

                Ok(Self::new(change_id))
            }
        }
    }

    /// Save annotations to a file using atomic write
    ///
    /// Writes to a temporary file first, then renames to prevent corruption.
    /// Uses tempfile::NamedTempFile::persist for cross-platform atomic replacement.
    pub fn save(&self, path: &Path) -> AnnotationResult<()> {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = serde_json::to_string_pretty(self)?;

        // Ensure parent directory exists
        let parent = path.parent().ok_or_else(|| {
            AnnotationError::InvalidPath("path has no parent directory".to_string())
        })?;
        fs::create_dir_all(parent)?;

        // Atomic write: create temp file in same directory, then persist (atomic replace)
        let mut temp_file = NamedTempFile::new_in(parent)?;
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;

        // persist() handles cross-platform atomic replacement (works on Windows too)
        temp_file.persist(path).map_err(|e| {
            AnnotationError::ReadError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to persist temp file: {}", e),
            ))
        })?;

        Ok(())
    }

    /// Add a new annotation
    pub fn add(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Find an annotation by ID
    pub fn find(&self, id: &str) -> Option<&Annotation> {
        self.annotations.iter().find(|a| a.id == id)
    }

    /// Find an annotation by ID (mutable)
    pub fn find_mut(&mut self, id: &str) -> Option<&mut Annotation> {
        self.annotations.iter_mut().find(|a| a.id == id)
    }

    /// Get all annotations for a specific file
    pub fn for_file(&self, file: &str) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.file == file).collect()
    }

    /// Get all annotations for a specific file and section
    pub fn for_section(&self, file: &str, section_id: &str) -> Vec<&Annotation> {
        self.annotations
            .iter()
            .filter(|a| a.file == file && a.section_id == section_id)
            .collect()
    }

    /// Mark an annotation as resolved
    pub fn resolve(&mut self, id: &str) -> AnnotationResult<()> {
        let annotation = self
            .find_mut(id)
            .ok_or_else(|| AnnotationError::NotFound(id.to_string()))?;
        annotation.resolve();
        Ok(())
    }

    /// Remove an annotation by ID
    pub fn remove(&mut self, id: &str) -> AnnotationResult<Annotation> {
        let pos = self
            .annotations
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| AnnotationError::NotFound(id.to_string()))?;
        Ok(self.annotations.remove(pos))
    }

    /// Get the number of annotations
    pub fn len(&self) -> usize {
        self.annotations.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Get count of unresolved annotations
    pub fn unresolved_count(&self) -> usize {
        self.annotations.iter().filter(|a| !a.resolved).count()
    }
}

/// Get the current user's name for annotation authorship
///
/// Priority:
/// 1. Git config user.name
/// 2. OS username from environment
/// 3. "unknown"
pub fn get_author_name() -> String {
    // Try git config first
    if let Ok(repo) = git2::Repository::discover(".") {
        if let Ok(config) = repo.config() {
            if let Ok(name) = config.get_string("user.name") {
                return name;
            }
        }
    }

    // Try environment variables
    if let Ok(user) = std::env::var("USER") {
        return user;
    }
    if let Ok(user) = std::env::var("USERNAME") {
        return user;
    }

    // Fallback
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_annotation_new() {
        let annotation = Annotation::new(
            "proposal.md",
            "r1-native-window",
            "Use tao 0.30+",
            "test-user",
        );

        assert_eq!(annotation.file, "proposal.md");
        assert_eq!(annotation.section_id, "r1-native-window");
        assert_eq!(annotation.content, "Use tao 0.30+");
        assert_eq!(annotation.author, "test-user");
        assert!(!annotation.resolved);

        // Verify UUID format
        assert!(Uuid::parse_str(&annotation.id).is_ok());

        // Verify timestamp is valid ISO 8601
        assert!(DateTime::parse_from_rfc3339(&annotation.created_at).is_ok());
    }

    #[test]
    fn test_annotation_resolve() {
        let mut annotation = Annotation::new("proposal.md", "r1", "comment", "user");
        assert!(!annotation.resolved);

        annotation.resolve();
        assert!(annotation.resolved);
    }

    #[test]
    fn test_annotation_store_new() {
        let store = AnnotationStore::new("my-change");
        assert_eq!(store.change_id, "my-change");
        assert!(store.is_empty());
    }

    #[test]
    fn test_annotation_store_add_and_find() {
        let mut store = AnnotationStore::new("my-change");
        let annotation = Annotation::new("proposal.md", "r1", "comment", "user");
        let id = annotation.id.clone();

        store.add(annotation);
        assert_eq!(store.len(), 1);

        let found = store.find(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().content, "comment");
    }

    #[test]
    fn test_annotation_store_for_file() {
        let mut store = AnnotationStore::new("my-change");
        store.add(Annotation::new("proposal.md", "r1", "comment1", "user"));
        store.add(Annotation::new("proposal.md", "r2", "comment2", "user"));
        store.add(Annotation::new("CHALLENGE.md", "issue1", "comment3", "user"));

        let proposal_annotations = store.for_file("proposal.md");
        assert_eq!(proposal_annotations.len(), 2);

        let challenge_annotations = store.for_file("CHALLENGE.md");
        assert_eq!(challenge_annotations.len(), 1);
    }

    #[test]
    fn test_annotation_store_for_section() {
        let mut store = AnnotationStore::new("my-change");
        store.add(Annotation::new("proposal.md", "r1", "comment1", "user"));
        store.add(Annotation::new("proposal.md", "r1", "comment2", "user"));
        store.add(Annotation::new("proposal.md", "r2", "comment3", "user"));

        let r1_annotations = store.for_section("proposal.md", "r1");
        assert_eq!(r1_annotations.len(), 2);

        let r2_annotations = store.for_section("proposal.md", "r2");
        assert_eq!(r2_annotations.len(), 1);
    }

    #[test]
    fn test_annotation_store_resolve() {
        let mut store = AnnotationStore::new("my-change");
        let annotation = Annotation::new("proposal.md", "r1", "comment", "user");
        let id = annotation.id.clone();
        store.add(annotation);

        assert_eq!(store.unresolved_count(), 1);

        store.resolve(&id).unwrap();
        assert_eq!(store.unresolved_count(), 0);

        let found = store.find(&id).unwrap();
        assert!(found.resolved);
    }

    #[test]
    fn test_annotation_store_remove() {
        let mut store = AnnotationStore::new("my-change");
        let annotation = Annotation::new("proposal.md", "r1", "comment", "user");
        let id = annotation.id.clone();
        store.add(annotation);

        assert_eq!(store.len(), 1);

        let removed = store.remove(&id).unwrap();
        assert_eq!(removed.content, "comment");
        assert!(store.is_empty());
    }

    #[test]
    fn test_annotation_store_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        let mut store = AnnotationStore::new("my-change");
        store.add(Annotation::new("proposal.md", "r1", "comment1", "user"));
        store.add(Annotation::new("proposal.md", "r2", "comment2", "user"));

        store.save(&path).unwrap();

        let loaded = AnnotationStore::load(&path, "my-change").unwrap();
        assert_eq!(loaded.change_id, "my-change");
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_annotation_store_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        let store = AnnotationStore::load(&path, "my-change").unwrap();
        assert_eq!(store.change_id, "my-change");
        assert!(store.is_empty());
    }

    #[test]
    fn test_annotation_store_load_malformed() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        // Write malformed JSON
        fs::write(&path, "{ invalid json }").unwrap();

        let store = AnnotationStore::load(&path, "my-change").unwrap();
        assert!(store.is_empty());

        // Verify backup was created
        let backup_path = path.with_extension("json.bak");
        assert!(backup_path.exists());
    }

    #[test]
    fn test_annotation_serialization() {
        let annotation = Annotation::new("proposal.md", "r1", "comment", "user");

        let json = serde_json::to_string(&annotation).unwrap();
        let parsed: Annotation = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.file, annotation.file);
        assert_eq!(parsed.section_id, annotation.section_id);
        assert_eq!(parsed.content, annotation.content);
        assert_eq!(parsed.author, annotation.author);
        assert_eq!(parsed.id, annotation.id);
        assert_eq!(parsed.created_at, annotation.created_at);
        assert_eq!(parsed.resolved, annotation.resolved);
    }

    #[test]
    fn test_annotation_store_serialization() {
        let mut store = AnnotationStore::new("my-change");
        store.add(Annotation::new("proposal.md", "r1", "comment", "user"));

        let json = serde_json::to_string_pretty(&store).unwrap();

        // Verify JSON structure matches spec
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
    fn test_get_author_name_fallback() {
        // This test may return different values depending on environment
        // Just verify it returns something non-empty
        let author = get_author_name();
        assert!(!author.is_empty());
    }

    #[test]
    fn test_annotation_uuid_uniqueness() {
        let a1 = Annotation::new("proposal.md", "r1", "comment1", "user");
        let a2 = Annotation::new("proposal.md", "r1", "comment2", "user");

        assert_ne!(a1.id, a2.id);
    }

    #[test]
    fn test_annotation_store_multiple_saves() {
        // Tests that save works multiple times (addresses Windows fs::rename issue)
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("annotations.json");

        let mut store = AnnotationStore::new("my-change");

        // First save (file doesn't exist yet)
        store.add(Annotation::new("proposal.md", "r1", "comment1", "user"));
        store.save(&path).unwrap();

        // Second save (file exists - should overwrite atomically)
        store.add(Annotation::new("proposal.md", "r2", "comment2", "user"));
        store.save(&path).unwrap();

        // Third save (verify still works)
        store.add(Annotation::new("proposal.md", "r3", "comment3", "user"));
        store.save(&path).unwrap();

        // Verify all data was persisted
        let loaded = AnnotationStore::load(&path, "my-change").unwrap();
        assert_eq!(loaded.len(), 3);
    }

    #[test]
    fn test_annotation_store_save_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nested/dir/annotations.json");

        let mut store = AnnotationStore::new("my-change");
        store.add(Annotation::new("proposal.md", "r1", "comment", "user"));

        // Should create parent directories
        store.save(&path).unwrap();
        assert!(path.exists());
    }
}
