//! generate_mermaid_state MCP Tool
//!
//! Generates Mermaid state diagrams for state machines, UI states, and workflows.

use super::super::{get_required_array, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_state
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_state".to_string(),
        description: "Generate a Mermaid state diagram from structured state and transition definitions. Use for state machines, UI state management, and workflow modeling.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["states", "transitions"],
            "properties": {
                "direction": {
                    "type": "string",
                    "enum": ["LR", "TB"],
                    "default": "TB",
                    "description": "Diagram direction: LR (left-right) or TB (top-bottom)"
                },
                "states": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["id", "label"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^[A-Za-z0-9_]+$",
                                "description": "Unique state identifier"
                            },
                            "label": {
                                "type": "string",
                                "description": "State display name"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["normal", "start", "end", "choice", "fork", "join"],
                                "default": "normal",
                                "description": "State type"
                            },
                            "description": {
                                "type": "string",
                                "description": "Optional state description"
                            }
                        }
                    },
                    "description": "List of states"
                },
                "transitions": {
                    "type": "array",
                    "minItems": 0,
                    "items": {
                        "type": "object",
                        "required": ["from", "to"],
                        "properties": {
                            "from": {
                                "type": "string",
                                "description": "Source state ID"
                            },
                            "to": {
                                "type": "string",
                                "description": "Target state ID"
                            },
                            "label": {
                                "type": "string",
                                "description": "Transition label/event"
                            }
                        }
                    },
                    "description": "List of transitions between states"
                },
                "composite_states": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "label", "substates"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Composite state identifier"
                            },
                            "label": {
                                "type": "string",
                                "description": "Composite state label"
                            },
                            "substates": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "IDs of states within this composite state"
                            }
                        }
                    },
                    "description": "Optional composite (nested) states"
                },
                "notes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["state", "text"],
                        "properties": {
                            "state": {
                                "type": "string",
                                "description": "State ID to attach note to"
                            },
                            "text": {
                                "type": "string",
                                "description": "Note text"
                            },
                            "position": {
                                "type": "string",
                                "enum": ["right", "left"],
                                "default": "right",
                                "description": "Note position"
                            }
                        }
                    },
                    "description": "Optional notes for states"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_state tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let states = get_required_array(args, "states")?;
    let transitions = get_required_array(args, "transitions")?;

    // Optional fields
    let direction = args
        .get("direction")
        .and_then(|v| v.as_str())
        .unwrap_or("TB");
    let composite_states = args
        .get("composite_states")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let notes = args
        .get("notes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Validate
    if states.is_empty() {
        anyhow::bail!("At least one state is required");
    }

    if !["LR", "TB"].contains(&direction) {
        anyhow::bail!("Invalid direction. Must be LR or TB");
    }

    // Generate Mermaid state diagram
    let mut mermaid = String::new();
    mermaid.push_str(&format!("stateDiagram-v2\n"));

    if direction == "LR" {
        mermaid.push_str("    direction LR\n");
    }

    // Track which states are in composite states
    let composite_state_members: std::collections::HashSet<String> = composite_states
        .iter()
        .filter_map(|cs| cs.get("substates").and_then(|v| v.as_array()))
        .flat_map(|substates| {
            substates
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    // Generate composite states
    for composite in &composite_states {
        let comp_id = composite
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Composite state missing 'id' field"))?;
        let comp_label = composite
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or(comp_id);
        let substates = composite
            .get("substates")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Composite state missing 'substates' array"))?;

        mermaid.push_str(&format!("    state \"{}\" as {} {{\n", comp_label, comp_id));

        // Generate substates
        for substate_id in substates {
            let substate_id_str = substate_id
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Substate ID must be a string"))?;

            // Find the state definition
            let state = states
                .iter()
                .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(substate_id_str))
                .ok_or_else(|| {
                    anyhow::anyhow!("State '{}' referenced in composite not found", substate_id_str)
                })?;

            let state_label = state
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or(substate_id_str);
            let state_type = state
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("normal");

            mermaid.push_str(&format_state(
                substate_id_str,
                state_label,
                state_type,
                "        ",
            )?);

            // Add state description if present
            if let Some(desc) = state.get("description").and_then(|v| v.as_str()) {
                mermaid.push_str(&format!("        {}: {}\n", substate_id_str, desc));
            }
        }

        mermaid.push_str("    }\n");
    }

    // Generate standalone states (not in composite states)
    for state in &states {
        let state_id = state
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("State missing 'id' field"))?;

        // Skip if already in a composite state
        if composite_state_members.contains(state_id) {
            continue;
        }

        let state_label = state
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or(state_id);
        let state_type = state
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");

        mermaid.push_str(&format_state(state_id, state_label, state_type, "    ")?);

        // Add state description if present
        if let Some(desc) = state.get("description").and_then(|v| v.as_str()) {
            mermaid.push_str(&format!("    {}: {}\n", state_id, desc));
        }
    }

    // Generate transitions
    for transition in &transitions {
        let from = transition
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Transition missing 'from' field"))?;
        let to = transition
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Transition missing 'to' field"))?;
        let label = transition.get("label").and_then(|v| v.as_str());

        if let Some(lbl) = label {
            mermaid.push_str(&format!("    {} --> {}: {}\n", from, to, lbl));
        } else {
            mermaid.push_str(&format!("    {} --> {}\n", from, to));
        }
    }

    // Generate notes
    for note in &notes {
        let state = note
            .get("state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Note missing 'state' field"))?;
        let text = note
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Note missing 'text' field"))?;
        let position = note
            .get("position")
            .and_then(|v| v.as_str())
            .unwrap_or("right");

        mermaid.push_str(&format!(
            "    note {} of {}\n        {}\n    end note\n",
            position, state, text
        ));
    }

    Ok(mermaid)
}

/// Format a state based on its type
fn format_state(id: &str, label: &str, state_type: &str, indent: &str) -> Result<String> {
    let state_str = match state_type {
        "normal" => {
            if id != label {
                format!("{}state \"{}\" as {}\n", indent, label, id)
            } else {
                format!("{}{}\n", indent, id)
            }
        }
        "start" => format!("{}[*] --> {}\n", indent, id),
        "end" => format!("{}{} --> [*]\n", indent, id),
        "choice" => format!("{}state {} <<choice>>\n", indent, id),
        "fork" => format!("{}state {} <<fork>>\n", indent, id),
        "join" => format!("{}state {} <<join>>\n", indent, id),
        _ => anyhow::bail!("Invalid state type: {}", state_type),
    };

    Ok(state_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_state_diagram() {
        let args = json!({
            "states": [
                {"id": "Idle", "label": "Idle State", "type": "normal"},
                {"id": "Processing", "label": "Processing", "type": "normal"},
                {"id": "Done", "label": "Done", "type": "normal"}
            ],
            "transitions": [
                {"from": "Idle", "to": "Processing", "label": "start"},
                {"from": "Processing", "to": "Done", "label": "complete"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("stateDiagram-v2"));
        assert!(result.contains("state \"Idle State\" as Idle"));
        assert!(result.contains("Idle --> Processing: start"));
        assert!(result.contains("Processing --> Done: complete"));
    }

    #[test]
    fn test_state_with_start_end() {
        let args = json!({
            "states": [
                {"id": "Start", "label": "Start", "type": "start"},
                {"id": "Middle", "label": "Middle", "type": "normal"},
                {"id": "End", "label": "End", "type": "end"}
            ],
            "transitions": [
                {"from": "Start", "to": "Middle"},
                {"from": "Middle", "to": "End"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("[*] --> Start"));
        assert!(result.contains("End --> [*]"));
    }

    #[test]
    fn test_composite_state() {
        let args = json!({
            "states": [
                {"id": "Init", "label": "Init", "type": "normal"},
                {"id": "Sub1", "label": "Sub 1", "type": "normal"},
                {"id": "Sub2", "label": "Sub 2", "type": "normal"},
                {"id": "Final", "label": "Final", "type": "normal"}
            ],
            "transitions": [
                {"from": "Init", "to": "Working"},
                {"from": "Working", "to": "Final"}
            ],
            "composite_states": [
                {
                    "id": "Working",
                    "label": "Working State",
                    "substates": ["Sub1", "Sub2"]
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("state \"Working State\" as Working"));
        assert!(result.contains("Sub1"));
        assert!(result.contains("Sub2"));
    }

    #[test]
    fn test_invalid_direction() {
        let args = json!({
            "direction": "INVALID",
            "states": [{"id": "A", "label": "Test"}],
            "transitions": []
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid direction"));
    }
}
