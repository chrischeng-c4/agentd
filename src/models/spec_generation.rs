use serde::{Deserialize, Serialize};

/// Request structure for spec generation via the Orchestrator
///
/// Used by the CodeStrategy to communicate with the LLM Orchestrator
/// when reverse-engineering specifications from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecGenerationRequest {
    /// List of source files to analyze
    pub files: Vec<SourceFile>,

    /// Additional prompt/context for the LLM
    pub prompt: String,
}

/// Represents a source file with its path and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// Relative path from project root
    pub path: String,

    /// File content
    pub content: String,
}

impl SpecGenerationRequest {
    /// Create a new spec generation request
    pub fn new(prompt: String) -> Self {
        Self {
            files: Vec::new(),
            prompt,
        }
    }

    /// Add a source file to the request
    pub fn add_file(&mut self, path: String, content: String) {
        self.files.push(SourceFile { path, content });
    }

    /// Get the total number of files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Get the total size of all file contents in bytes
    pub fn total_size(&self) -> usize {
        self.files.iter().map(|f| f.content.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_generation_request_creation() {
        let mut request = SpecGenerationRequest::new("Analyze these files".to_string());
        assert_eq!(request.file_count(), 0);
        assert_eq!(request.total_size(), 0);

        request.add_file("src/main.rs".to_string(), "fn main() {}".to_string());
        assert_eq!(request.file_count(), 1);
        assert!(request.total_size() > 0);
    }

    #[test]
    fn test_multiple_files() {
        let mut request = SpecGenerationRequest::new("Test prompt".to_string());
        request.add_file("file1.rs".to_string(), "content1".to_string());
        request.add_file("file2.rs".to_string(), "content2".to_string());

        assert_eq!(request.file_count(), 2);
        assert_eq!(request.total_size(), "content1".len() + "content2".len());
    }
}
