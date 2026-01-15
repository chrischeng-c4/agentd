use crate::fillback::strategy::ImportStrategy;
use crate::Result;
use async_trait::async_trait;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::path::Path;

/// GitHub Speckit import strategy
///
/// Parses GitHub Speckit markdown files and restructures them into Agentd specifications.
/// Speckit uses a structured markdown format with specific sections.
pub struct SpeckitStrategy;

impl SpeckitStrategy {
    pub fn new() -> Self {
        Self
    }

    /// Parse Speckit markdown file
    fn parse_markdown(&self, path: &Path) -> Result<SpeckitDocument> {
        let content = std::fs::read_to_string(path)?;

        let mut doc = SpeckitDocument {
            title: String::new(),
            overview: String::new(),
            requirements: Vec::new(),
            scenarios: Vec::new(),
        };

        let parser = Parser::new(&content);
        let mut current_section: Option<String> = None;
        let mut current_text = String::new();
        let mut in_heading = false;
        let mut heading_level = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = true;
                    heading_level = level as usize;
                }
                Event::End(TagEnd::Heading { .. }) => {
                    in_heading = false;
                    if heading_level == 1 && doc.title.is_empty() {
                        doc.title = current_text.trim().to_string();
                    } else if heading_level == 2 {
                        current_section = Some(current_text.trim().to_string());
                    } else if heading_level == 3 {
                        // This might be a requirement or scenario heading
                        if let Some(ref section) = current_section {
                            if section.to_lowercase().contains("requirement") {
                                doc.requirements.push(current_text.trim().to_string());
                            } else if section.to_lowercase().contains("scenario")
                                || section.to_lowercase().contains("acceptance")
                            {
                                doc.scenarios.push(current_text.trim().to_string());
                            }
                        }
                    }
                    current_text.clear();
                }
                Event::Text(text) => {
                    if in_heading {
                        current_text.push_str(&text);
                    } else if let Some(ref section) = current_section {
                        if section.to_lowercase().contains("overview")
                            || section.to_lowercase().contains("summary")
                        {
                            doc.overview.push_str(&text);
                        }
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !in_heading {
                        current_text.push('\n');
                    }
                }
                _ => {}
            }
        }

        Ok(doc)
    }

    /// Convert Speckit document to Agentd proposal markdown
    fn to_proposal_md(&self, doc: &SpeckitDocument, change_id: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Change: {}\n\n", change_id));

        output.push_str("## Summary\n");
        if !doc.overview.is_empty() {
            output.push_str(&format!("{}\n\n", doc.overview.trim()));
        } else if !doc.title.is_empty() {
            output.push_str(&format!("{}\n\n", doc.title));
        } else {
            output.push_str("(Imported from GitHub Speckit)\n\n");
        }

        output.push_str("## Why\n");
        output.push_str("(To be filled in)\n\n");

        output.push_str("## What Changes\n");
        output.push_str("(To be filled in)\n\n");

        output.push_str("## Impact\n");
        output.push_str("- **Affected Specs:** (To be determined)\n");
        output.push_str("- **Affected Code:** (To be determined)\n");
        output.push_str("- **Breaking Changes:** (To be determined)\n");

        output
    }

    /// Convert Speckit document to Agentd spec markdown
    fn to_spec_md(&self, doc: &SpeckitDocument) -> String {
        let mut output = String::new();

        let title = if !doc.title.is_empty() {
            &doc.title
        } else {
            "Imported Specification"
        };

        output.push_str(&format!("# Specification: {}\n\n", title));

        output.push_str("## Overview\n");
        if !doc.overview.is_empty() {
            output.push_str(&format!("{}\n\n", doc.overview.trim()));
        } else {
            output.push_str("(Imported from GitHub Speckit)\n\n");
        }

        if !doc.requirements.is_empty() {
            output.push_str("## Requirements\n\n");
            for (i, req) in doc.requirements.iter().enumerate() {
                output.push_str(&format!("### R{}: {}\n\n", i + 1, req));
            }
        }

        if !doc.scenarios.is_empty() {
            output.push_str("## Acceptance Criteria\n\n");
            for scenario in &doc.scenarios {
                output.push_str(&format!("### Scenario: {}\n", scenario));
                output.push_str("- **WHEN** (To be specified)\n");
                output.push_str("- **THEN** (To be specified)\n\n");
            }
        }

        output
    }
}

/// Speckit document structure parsed from markdown
struct SpeckitDocument {
    title: String,
    overview: String,
    requirements: Vec<String>,
    scenarios: Vec<String>,
}

#[async_trait]
impl ImportStrategy for SpeckitStrategy {
    async fn execute(&self, source: &Path, change_id: &str) -> Result<()> {
        // Parse the Speckit markdown file
        let doc = self.parse_markdown(source)?;

        // Create change directory
        let current_dir = std::env::current_dir()?;
        let change_dir = current_dir.join("agentd/changes").join(change_id);
        std::fs::create_dir_all(&change_dir)?;

        // Generate proposal.md
        let proposal_content = self.to_proposal_md(&doc, change_id);
        std::fs::write(change_dir.join("proposal.md"), proposal_content)?;

        // Generate spec file in specs/ directory
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir)?;

        let spec_content = self.to_spec_md(&doc);
        let spec_filename = if !doc.title.is_empty() {
            format!("{}.md", doc.title.to_lowercase().replace(' ', "_"))
        } else {
            "imported_spec.md".to_string()
        };
        std::fs::write(specs_dir.join(spec_filename), spec_content)?;

        // Generate tasks.md skeleton
        let tasks_content = "# Tasks\n\n## 1. Implementation\n- [ ] 1.1 Implement imported requirements\n  - File: (To be determined)\n  - Do: Review and implement the requirements from the imported spec.\n";
        std::fs::write(change_dir.join("tasks.md"), tasks_content)?;

        Ok(())
    }

    fn can_handle(&self, source: &Path) -> bool {
        if !source.is_file() {
            return false;
        }

        // Check file extension
        if let Some(ext) = source.extension().and_then(|s| s.to_str()) {
            if ext == "md" || ext == "markdown" {
                // Try to read and check if it looks like a Speckit file
                // Speckit files typically have headings and structured content
                if let Ok(content) = std::fs::read_to_string(source) {
                    // Basic heuristic: check for common Speckit sections
                    let lower = content.to_lowercase();
                    return lower.contains("# ")
                        && (lower.contains("requirement") || lower.contains("scenario"));
                }
            }
        }

        false
    }

    fn name(&self) -> &'static str {
        "speckit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_speckit_markdown_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let speckit_file = temp_dir.path().join("spec.md");

        let md_content = r#"# Test Specification

## Overview
This is a test specification for the system.

## Requirements

### User Authentication
The system must authenticate users.

### Data Validation
The system must validate all inputs.

## Acceptance Criteria

### Login Success
User can log in successfully.

### Invalid Input Handling
System rejects invalid inputs.
"#;
        std::fs::write(&speckit_file, md_content).unwrap();

        let strategy = SpeckitStrategy::new();
        let doc = strategy.parse_markdown(&speckit_file).unwrap();

        assert_eq!(doc.title, "Test Specification");
        assert!(doc.overview.contains("test specification"));
        assert!(doc.requirements.len() >= 1);
    }

    #[test]
    fn test_can_handle_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let speckit_file = temp_dir.path().join("spec.md");

        let md_content = "# Test\n\n## Requirements\n\nSome requirement";
        std::fs::write(&speckit_file, md_content).unwrap();

        let strategy = SpeckitStrategy::new();
        assert!(strategy.can_handle(&speckit_file));
    }

    #[test]
    fn test_can_handle_non_speckit() {
        let temp_dir = TempDir::new().unwrap();
        let readme = temp_dir.path().join("README.md");
        std::fs::write(&readme, "# Just a readme\n\nSome content").unwrap();

        let strategy = SpeckitStrategy::new();
        // This might return false since there's no "requirement" or "scenario"
        assert!(!strategy.can_handle(&readme));
    }
}
