use super::{get_required_array, get_required_string, ToolDefinition};
use crate::Result;
use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;

pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_clarifications".to_string(),
        description: "Create clarifications.md file with structured Q&A from user".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["change_id", "questions"],
            "properties": {
                "change_id": {
                    "type": "string",
                    "pattern": "^[a-z0-9-]+$",
                    "description": "Change ID (lowercase, hyphens allowed)"
                },
                "questions": {
                    "type": "array",
                    "minItems": 1,
                    "description": "Array of Q&A pairs",
                    "items": {
                        "type": "object",
                        "required": ["topic", "question", "answer", "rationale"],
                        "properties": {
                            "topic": {
                                "type": "string",
                                "description": "Short topic label (e.g., 'Authentication Method')"
                            },
                            "question": {
                                "type": "string",
                                "description": "The question asked to the user"
                            },
                            "answer": {
                                "type": "string",
                                "description": "User's answer"
                            },
                            "rationale": {
                                "type": "string",
                                "description": "Why this choice was made"
                            }
                        }
                    }
                }
            }
        }),
    }
}

pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let questions = get_required_array(args, "questions")?;

    // Validate change_id format (security: prevent directory traversal)
    if !change_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }

    // Create change directory if needed
    let change_dir = project_root.join("agentd/changes").join(&change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }

    let clarifications_path = change_dir.join("clarifications.md");

    // Generate frontmatter
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut content = format!("---\nchange: {}\ndate: {}\n---\n\n# Clarifications\n\n", change_id, today);

    // Generate Q&A sections
    for (i, qa) in questions.iter().enumerate() {
        let topic = qa.get("topic").and_then(|v| v.as_str()).unwrap_or("General");
        let question = qa.get("question").and_then(|v| v.as_str()).unwrap_or("");
        let answer = qa.get("answer").and_then(|v| v.as_str()).unwrap_or("");
        let rationale = qa.get("rationale").and_then(|v| v.as_str()).unwrap_or("");

        content.push_str(&format!("## Q{}: {}\n", i + 1, topic));
        content.push_str(&format!("- **Question**: {}\n", question));
        content.push_str(&format!("- **Answer**: {}\n", answer));
        content.push_str(&format!("- **Rationale**: {}\n", rationale));
        content.push('\n');
    }

    std::fs::write(&clarifications_path, content)?;

    Ok(format!(
        "✓ Clarifications written: agentd/changes/{}/clarifications.md\n  Questions: {}",
        change_id,
        questions.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_create_clarifications_valid() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "add-oauth",
            "questions": [
                {
                    "topic": "Auth Method",
                    "question": "Which OAuth providers?",
                    "answer": "Google and GitHub",
                    "rationale": "Most common enterprise providers"
                }
            ]
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("✓ Clarifications written"));

        let file_path = project_root.join("agentd/changes/add-oauth/clarifications.md");
        assert!(file_path.exists());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("change: add-oauth"));
        assert!(content.contains("## Q1: Auth Method"));
        assert!(content.contains("**Question**: Which OAuth providers?"));
    }

    #[test]
    fn test_create_clarifications_invalid_change_id() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "../etc/passwd",
            "questions": [{"topic": "test", "question": "test", "answer": "test", "rationale": "test"}]
        });

        let result = execute(&args, project_root);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_clarifications_multiple_questions() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let args = json!({
            "change_id": "multi-test",
            "questions": [
                {
                    "topic": "First Topic",
                    "question": "First question?",
                    "answer": "First answer",
                    "rationale": "First rationale"
                },
                {
                    "topic": "Second Topic",
                    "question": "Second question?",
                    "answer": "Second answer",
                    "rationale": "Second rationale"
                }
            ]
        });

        let result = execute(&args, project_root).unwrap();
        assert!(result.contains("Questions: 2"));

        let file_path = project_root.join("agentd/changes/multi-test/clarifications.md");
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("## Q1: First Topic"));
        assert!(content.contains("## Q2: Second Topic"));
    }
}
