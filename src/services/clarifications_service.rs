//! Clarifications service - Business logic for creating clarifications.md
//!
//! Provides structured Q&A capture from user interactions during planning.

use crate::models::frontmatter::StatePhase;
use crate::state::StateManager;
use crate::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionAnswer {
    pub topic: String,
    pub question: String,
    pub answer: String,
    pub rationale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClarificationsInput {
    pub change_id: String,
    pub questions: Vec<QuestionAnswer>,
}

/// Create clarifications.md file with structured Q&A
pub fn create_clarifications(input: CreateClarificationsInput, project_root: &Path) -> Result<String> {
    let change_id = &input.change_id;

    // Validate change_id format (security: prevent directory traversal)
    if !change_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    // Create change directory if needed
    let change_dir = project_root.join("agentd/changes").join(change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let clarifications_path = change_dir.join("clarifications.md");

    // Generate frontmatter
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut content = format!("---\nchange: {}\ndate: {}\n---\n\n# Clarifications\n\n", change_id, today);

    // Generate Q&A sections
    for (i, qa) in input.questions.iter().enumerate() {
        content.push_str(&format!("## Q{}: {}\n", i + 1, qa.topic));
        content.push_str(&format!("- **Question**: {}\n", qa.question));
        content.push_str(&format!("- **Answer**: {}\n", qa.answer));
        content.push_str(&format!("- **Rationale**: {}\n", qa.rationale));
        content.push('\n');
    }

    std::fs::write(&clarifications_path, content)?;

    // Initialize STATE.yaml if it doesn't exist
    let state_path = change_dir.join("STATE.yaml");
    if !state_path.exists() {
        let mut state_manager = StateManager::load(&change_dir)?;
        state_manager.set_phase(StatePhase::Proposed);
        state_manager.set_last_action("clarifications created");
        state_manager.save()?;
    }

    Ok(format!(
        "✓ Clarifications written: agentd/changes/{}/clarifications.md\n  Questions: {}",
        change_id,
        input.questions.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_clarifications_valid() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let input = CreateClarificationsInput {
            change_id: "add-oauth".to_string(),
            questions: vec![QuestionAnswer {
                topic: "Auth Method".to_string(),
                question: "Which OAuth providers?".to_string(),
                answer: "Google and GitHub".to_string(),
                rationale: "Most common enterprise providers".to_string(),
            }],
        };

        let result = create_clarifications(input, project_root).unwrap();
        assert!(result.contains("✓ Clarifications written"));

        let file_path = project_root.join("agentd/changes/add-oauth/clarifications.md");
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("change: add-oauth"));
        assert!(content.contains("## Q1: Auth Method"));
        assert!(content.contains("**Question**: Which OAuth providers?"));

        // Verify STATE.yaml was created
        let state_path = project_root.join("agentd/changes/add-oauth/STATE.yaml");
        assert!(state_path.exists(), "STATE.yaml should be created");
        let state_content = std::fs::read_to_string(&state_path).unwrap();
        assert!(state_content.contains("phase: proposed"));
        assert!(state_content.contains("last_action: clarifications created"));
    }

    #[test]
    fn test_create_clarifications_invalid_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let input = CreateClarificationsInput {
            change_id: "../etc/passwd".to_string(),
            questions: vec![QuestionAnswer {
                topic: "test".to_string(),
                question: "test".to_string(),
                answer: "test".to_string(),
                rationale: "test".to_string(),
            }],
        };

        let result = create_clarifications(input, project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_clarifications_multiple_questions() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let input = CreateClarificationsInput {
            change_id: "multi-test".to_string(),
            questions: vec![
                QuestionAnswer {
                    topic: "First Topic".to_string(),
                    question: "First question?".to_string(),
                    answer: "First answer".to_string(),
                    rationale: "First rationale".to_string(),
                },
                QuestionAnswer {
                    topic: "Second Topic".to_string(),
                    question: "Second question?".to_string(),
                    answer: "Second answer".to_string(),
                    rationale: "Second rationale".to_string(),
                },
            ],
        };

        let result = create_clarifications(input, project_root).unwrap();
        assert!(result.contains("Questions: 2"));

        let file_path = project_root.join("agentd/changes/multi-test/clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Q1: First Topic"));
        assert!(content.contains("## Q2: Second Topic"));
    }
}
