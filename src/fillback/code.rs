use crate::fillback::strategy::ImportStrategy;
use crate::models::{SourceFile, SpecGenerationRequest};
use crate::orchestrator::ScriptRunner;
use crate::Result;
use async_trait::async_trait;
use colored::Colorize;
use ignore::WalkBuilder;
use std::path::Path;

/// Code import strategy
///
/// Analyzes source code files and uses the LLM Orchestrator to generate
/// high-level technical designs and specifications.
pub struct CodeStrategy;

impl CodeStrategy {
    pub fn new() -> Self {
        Self
    }

    /// Scan source directory and collect files
    ///
    /// Uses the `ignore` crate to respect .gitignore patterns
    fn scan_files(&self, source: &Path) -> Result<Vec<SourceFile>> {
        let mut files = Vec::new();
        let max_files = 100; // Limit to prevent overwhelming the LLM
        let max_file_size = 50_000; // 50KB limit per file

        let walker = WalkBuilder::new(source)
            .standard_filters(true) // Respect .gitignore, .ignore, .git/info/exclude
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            // Only process files
            if !path.is_file() {
                continue;
            }

            // Skip very large files
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > max_file_size as u64 {
                    continue;
                }
            }

            // Only process source code files (basic heuristic)
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let is_source = matches!(
                    ext,
                    "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" |
                    "h" | "hpp" | "cs" | "rb" | "php" | "swift" | "kt" | "scala" | "sh" |
                    "toml" | "yaml" | "yml" | "json"
                );

                if !is_source {
                    continue;
                }
            } else {
                continue;
            }

            // Read file content
            if let Ok(content) = std::fs::read_to_string(path) {
                let relative_path = path
                    .strip_prefix(source)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                files.push(SourceFile {
                    path: relative_path,
                    content,
                });

                // Stop if we've collected enough files
                if files.len() >= max_files {
                    println!(
                        "{}",
                        format!(
                            "⚠️  Reached file limit ({}). Some files were skipped.",
                            max_files
                        )
                        .yellow()
                    );
                    break;
                }
            }
        }

        Ok(files)
    }

    /// Call the orchestrator to generate specifications
    async fn generate_specs(
        &self,
        request: &SpecGenerationRequest,
        change_id: &str,
    ) -> Result<()> {
        println!(
            "{}",
            format!(
                "📊 Analyzing {} files ({} bytes total)",
                request.file_count(),
                request.total_size()
            )
            .cyan()
        );

        // Use ScriptRunner to call the fillback script
        let current_dir = std::env::current_dir()?;
        let scripts_dir = current_dir.join("agentd/scripts");

        let runner = ScriptRunner::new(scripts_dir);

        // Prepare args: change_id and JSON request
        let request_json = serde_json::to_string(request)?;
        let args = vec![change_id.to_string(), request_json];

        // Run the gemini-fillback.sh script
        let output = runner
            .run_script("gemini-fillback.sh", &args, true)
            .await?;

        println!("{}", "✅ Spec generation completed".green());
        println!("{}", output);

        Ok(())
    }
}

#[async_trait]
impl ImportStrategy for CodeStrategy {
    async fn execute(&self, source: &Path, change_id: &str) -> Result<()> {
        println!(
            "{}",
            format!("🔍 Scanning codebase at: {}", source.display()).cyan()
        );

        // Scan files
        let files = self.scan_files(source)?;

        if files.is_empty() {
            anyhow::bail!("No source files found in: {}", source.display());
        }

        println!(
            "{}",
            format!("📁 Found {} source files", files.len()).cyan()
        );

        // Create change directory
        let current_dir = std::env::current_dir()?;
        let change_dir = current_dir.join("agentd/changes").join(change_id);
        std::fs::create_dir_all(&change_dir)?;

        // Create specs directory
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir)?;

        // Build spec generation request
        let mut request = SpecGenerationRequest::new(
            "Reverse-engineer technical specifications from this codebase. \
             Analyze the code structure, identify key components, and generate \
             comprehensive Agentd specifications including requirements and scenarios."
                .to_string(),
        );

        for file in files {
            request.add_file(file.path, file.content);
        }

        // Call orchestrator to generate specs
        self.generate_specs(&request, change_id).await?;

        // Ensure skeleton files exist if orchestrator didn't create them
        self.ensure_skeleton_files(&change_dir, change_id)?;

        Ok(())
    }

    fn can_handle(&self, source: &Path) -> bool {
        // Code strategy can handle any directory
        source.is_dir()
    }

    fn name(&self) -> &'static str {
        "code"
    }
}

impl CodeStrategy {
    /// Ensure skeleton files exist for proposal and tasks
    fn ensure_skeleton_files(&self, change_dir: &Path, change_id: &str) -> Result<()> {
        // Check if proposal.md exists
        if !change_dir.join("proposal.md").exists() {
            let proposal_content = format!(
                "# Change: {}\n\n\
                 ## Summary\n\
                 (Generated from code analysis)\n\n\
                 ## Why\n\
                 (To be filled in)\n\n\
                 ## What Changes\n\
                 (To be filled in)\n\n\
                 ## Impact\n\
                 - **Affected Specs:** (To be determined)\n\
                 - **Affected Code:** (To be determined)\n\
                 - **Breaking Changes:** (To be determined)\n",
                change_id
            );
            std::fs::write(change_dir.join("proposal.md"), proposal_content)?;
        }

        // Check if tasks.md exists
        if !change_dir.join("tasks.md").exists() {
            let tasks_content = "# Tasks\n\n\
                ## 1. Implementation\n\
                - [ ] 1.1 Review generated specifications\n\
                  - File: agentd/changes/*/specs/*.md\n\
                  - Do: Review and validate the auto-generated specifications.\n\
                - [ ] 1.2 Implement changes based on specifications\n\
                  - File: (To be determined)\n\
                  - Do: Implement the required changes.\n";
            std::fs::write(change_dir.join("tasks.md"), tasks_content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create some test files
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
        fs::write(src_dir.join("lib.rs"), "pub mod test;").unwrap();
        fs::write(src_dir.join("README.md"), "# Readme").unwrap(); // Should be skipped

        let strategy = CodeStrategy::new();
        let files = strategy.scan_files(&src_dir).unwrap();

        // Should find the .rs files but not .md
        assert!(files.len() >= 2);
        assert!(files.iter().any(|f| f.path.contains("main.rs")));
        assert!(files.iter().any(|f| f.path.contains("lib.rs")));
    }

    #[test]
    fn test_can_handle_directory() {
        let temp_dir = TempDir::new().unwrap();
        let strategy = CodeStrategy::new();

        assert!(strategy.can_handle(temp_dir.path()));
    }

    #[test]
    fn test_can_handle_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.rs");
        fs::write(&file, "fn test() {}").unwrap();

        let strategy = CodeStrategy::new();
        assert!(!strategy.can_handle(&file));
    }

    #[test]
    fn test_ensure_skeleton_files() {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join("test-change");
        fs::create_dir(&change_dir).unwrap();

        let strategy = CodeStrategy::new();
        strategy
            .ensure_skeleton_files(&change_dir, "test-change")
            .unwrap();

        assert!(change_dir.join("proposal.md").exists());
        assert!(change_dir.join("tasks.md").exists());
    }
}
