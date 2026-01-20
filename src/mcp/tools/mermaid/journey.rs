//! generate_mermaid_journey MCP Tool
//!
//! Generates Mermaid user journey diagrams for UX flows and service blueprints.

use super::super::{get_required_array, get_required_string, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_journey
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_journey".to_string(),
        description: "Generate a Mermaid user journey diagram from structured task and actor definitions. Use for UX flows, service blueprints, customer journey mapping, and satisfaction analysis.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["title", "sections"],
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Journey title"
                },
                "sections": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["name", "tasks"],
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Section name (e.g., 'Home', 'Shopping', 'Checkout')"
                            },
                            "tasks": {
                                "type": "array",
                                "minItems": 1,
                                "items": {
                                    "type": "object",
                                    "required": ["name", "actors", "score"],
                                    "properties": {
                                        "name": {
                                            "type": "string",
                                            "description": "Task name"
                                        },
                                        "actors": {
                                            "type": "array",
                                            "minItems": 1,
                                            "items": {
                                                "type": "string"
                                            },
                                            "description": "List of actors involved in this task"
                                        },
                                        "score": {
                                            "type": "integer",
                                            "minimum": 1,
                                            "maximum": 5,
                                            "description": "Satisfaction score (1-5)"
                                        }
                                    }
                                },
                                "description": "Tasks in this section"
                            }
                        }
                    },
                    "description": "Journey sections"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_journey tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let title = get_required_string(args, "title")?;
    let sections = get_required_array(args, "sections")?;

    // Validate
    if sections.is_empty() {
        anyhow::bail!("At least one section is required");
    }

    // Collect all unique actors
    let mut all_actors = std::collections::HashSet::new();
    for section in &sections {
        let tasks = section
            .get("tasks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Section missing 'tasks' array"))?;

        for task in tasks {
            let actors = task
                .get("actors")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Task missing 'actors' array"))?;

            for actor in actors {
                if let Some(actor_str) = actor.as_str() {
                    all_actors.insert(actor_str.to_string());
                }
            }
        }
    }

    // Generate Mermaid user journey
    let mut mermaid = String::new();
    mermaid.push_str(&format!("journey\n    title {}\n", title));

    // Generate sections
    for section in &sections {
        let section_name = section
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Section missing 'name' field"))?;

        mermaid.push_str(&format!("    section {}\n", section_name));

        let tasks = section
            .get("tasks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Section missing 'tasks' array"))?;

        if tasks.is_empty() {
            anyhow::bail!("Section '{}' has no tasks", section_name);
        }

        // Generate tasks
        for task in tasks {
            let task_name = task
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Task missing 'name' field"))?;
            let actors = task
                .get("actors")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Task missing 'actors' array"))?;
            let score = task
                .get("score")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Task missing 'score' field"))?;

            // Validate score
            if !(1..=5).contains(&score) {
                anyhow::bail!("Task '{}' has invalid score: {}. Must be 1-5", task_name, score);
            }

            if actors.is_empty() {
                anyhow::bail!("Task '{}' has no actors", task_name);
            }

            // Format actors
            let actors_str: Vec<String> = actors
                .iter()
                .filter_map(|a| a.as_str())
                .map(|s| s.to_string())
                .collect();

            mermaid.push_str(&format!(
                "      {}: {}: {}\n",
                task_name,
                score,
                actors_str.join(", ")
            ));
        }
    }

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_journey() {
        let args = json!({
            "title": "Online Shopping Experience",
            "sections": [
                {
                    "name": "Browse",
                    "tasks": [
                        {
                            "name": "View products",
                            "actors": ["Customer"],
                            "score": 5
                        },
                        {
                            "name": "Filter results",
                            "actors": ["Customer"],
                            "score": 4
                        }
                    ]
                },
                {
                    "name": "Purchase",
                    "tasks": [
                        {
                            "name": "Add to cart",
                            "actors": ["Customer"],
                            "score": 5
                        },
                        {
                            "name": "Complete checkout",
                            "actors": ["Customer", "Payment System"],
                            "score": 3
                        }
                    ]
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("journey"));
        assert!(result.contains("title Online Shopping Experience"));
        assert!(result.contains("section Browse"));
        assert!(result.contains("View products: 5: Customer"));
        assert!(result.contains("Filter results: 4: Customer"));
        assert!(result.contains("section Purchase"));
        assert!(result.contains("Add to cart: 5: Customer"));
        assert!(result.contains("Complete checkout: 3: Customer, Payment System"));
    }

    #[test]
    fn test_multiple_actors() {
        let args = json!({
            "title": "Support Ticket Flow",
            "sections": [
                {
                    "name": "Initial Contact",
                    "tasks": [
                        {
                            "name": "Submit ticket",
                            "actors": ["User", "Support System"],
                            "score": 4
                        }
                    ]
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("Submit ticket: 4: User, Support System"));
    }

    #[test]
    fn test_invalid_score() {
        let args = json!({
            "title": "Test",
            "sections": [
                {
                    "name": "Section1",
                    "tasks": [
                        {
                            "name": "Task1",
                            "actors": ["Actor1"],
                            "score": 10
                        }
                    ]
                }
            ]
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid score"));
    }

    #[test]
    fn test_empty_sections() {
        let args = json!({
            "title": "Test",
            "sections": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least one section is required"));
    }

    #[test]
    fn test_empty_tasks() {
        let args = json!({
            "title": "Test",
            "sections": [
                {
                    "name": "Section1",
                    "tasks": []
                }
            ]
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("has no tasks"));
    }

    #[test]
    fn test_empty_actors() {
        let args = json!({
            "title": "Test",
            "sections": [
                {
                    "name": "Section1",
                    "tasks": [
                        {
                            "name": "Task1",
                            "actors": [],
                            "score": 3
                        }
                    ]
                }
            ]
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("has no actors"));
    }
}
