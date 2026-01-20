//! generate_mermaid_sequence MCP Tool
//!
//! Generates Mermaid sequence diagrams for API calls, service interactions, and event flows.

use super::super::{get_required_array, ToolDefinition};
use crate::Result;
use serde_json::{json, Value};

/// Get the tool definition for generate_mermaid_sequence
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "generate_mermaid_sequence".to_string(),
        description: "Generate a Mermaid sequence diagram from structured participant and message definitions. Use for API interactions, service flows, and event sequences.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["participants", "messages"],
            "properties": {
                "participants": {
                    "type": "array",
                    "minItems": 2,
                    "items": {
                        "type": "object",
                        "required": ["id", "label"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^[A-Za-z0-9_]+$",
                                "description": "Unique participant identifier"
                            },
                            "label": {
                                "type": "string",
                                "description": "Participant display name"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["participant", "actor"],
                                "default": "participant",
                                "description": "Participant type"
                            }
                        }
                    },
                    "description": "List of participants in the sequence"
                },
                "messages": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["from", "to", "text"],
                        "properties": {
                            "from": {
                                "type": "string",
                                "description": "Source participant ID"
                            },
                            "to": {
                                "type": "string",
                                "description": "Target participant ID"
                            },
                            "text": {
                                "type": "string",
                                "description": "Message text"
                            },
                            "type": {
                                "type": "string",
                                "enum": ["solid", "dotted", "solid_open", "dotted_open"],
                                "default": "solid",
                                "description": "Arrow type: solid (->), dotted (-->), solid_open (->>), dotted_open (-->>)"
                            },
                            "activate": {
                                "type": "boolean",
                                "default": false,
                                "description": "Activate the target participant"
                            },
                            "deactivate": {
                                "type": "boolean",
                                "default": false,
                                "description": "Deactivate the target participant"
                            }
                        }
                    },
                    "description": "List of messages exchanged"
                },
                "notes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["text"],
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "Note text"
                            },
                            "position": {
                                "type": "string",
                                "enum": ["right_of", "left_of", "over"],
                                "default": "right_of",
                                "description": "Note position relative to participant"
                            },
                            "participant": {
                                "type": "string",
                                "description": "Participant ID for positioning"
                            },
                            "participants": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Multiple participant IDs (for 'over' position)"
                            },
                            "after_message": {
                                "type": "integer",
                                "description": "Insert note after this message index (0-based)"
                            }
                        }
                    },
                    "description": "Optional notes in the sequence"
                },
                "loops": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["label", "start_message", "end_message"],
                        "properties": {
                            "label": {
                                "type": "string",
                                "description": "Loop condition text"
                            },
                            "start_message": {
                                "type": "integer",
                                "description": "Start message index (0-based)"
                            },
                            "end_message": {
                                "type": "integer",
                                "description": "End message index (0-based)"
                            }
                        }
                    },
                    "description": "Optional loop blocks"
                },
                "alts": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["condition", "start_message", "end_message"],
                        "properties": {
                            "condition": {
                                "type": "string",
                                "description": "Alternative condition text"
                            },
                            "start_message": {
                                "type": "integer",
                                "description": "Start message index (0-based)"
                            },
                            "end_message": {
                                "type": "integer",
                                "description": "End message index (0-based)"
                            },
                            "else_condition": {
                                "type": "string",
                                "description": "Else condition text"
                            },
                            "else_end_message": {
                                "type": "integer",
                                "description": "Else block end message index"
                            }
                        }
                    },
                    "description": "Optional alternative (alt/else) blocks"
                }
            }
        }),
    }
}

/// Execute the generate_mermaid_sequence tool
pub fn execute(args: &Value) -> Result<String> {
    // Extract required fields
    let participants = get_required_array(args, "participants")?;
    let messages = get_required_array(args, "messages")?;

    // Optional fields
    let notes = args
        .get("notes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let loops = args
        .get("loops")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let alts = args
        .get("alts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Validate
    if participants.len() < 2 {
        anyhow::bail!("At least two participants are required");
    }

    if messages.is_empty() {
        anyhow::bail!("At least one message is required");
    }

    // Generate Mermaid sequence diagram
    let mut mermaid = String::new();
    mermaid.push_str("sequenceDiagram\n");

    // Generate participants
    for participant in &participants {
        let p_id = participant
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Participant missing 'id' field"))?;
        let p_label = participant
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or(p_id);
        let p_type = participant
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("participant");

        match p_type {
            "actor" => mermaid.push_str(&format!("    actor {} as {}\n", p_id, p_label)),
            "participant" => {
                mermaid.push_str(&format!("    participant {} as {}\n", p_id, p_label))
            }
            _ => anyhow::bail!("Invalid participant type: {}", p_type),
        }
    }

    // Process messages with blocks (loops, alts)
    let mut current_loop: Option<usize> = None;
    let mut current_alt: Option<usize> = None;

    for (msg_idx, message) in messages.iter().enumerate() {
        // Check if we need to start a loop
        for (loop_idx, loop_def) in loops.iter().enumerate() {
            let start = loop_def
                .get("start_message")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Loop missing 'start_message'"))?
                as usize;

            if msg_idx == start {
                let label = loop_def
                    .get("label")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Loop missing 'label'"))?;
                mermaid.push_str(&format!("    loop {}\n", label));
                current_loop = Some(loop_idx);
            }
        }

        // Check if we need to start an alt
        for (alt_idx, alt_def) in alts.iter().enumerate() {
            let start = alt_def
                .get("start_message")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Alt missing 'start_message'"))?
                as usize;

            if msg_idx == start {
                let condition = alt_def
                    .get("condition")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Alt missing 'condition'"))?;
                mermaid.push_str(&format!("    alt {}\n", condition));
                current_alt = Some(alt_idx);
            }

            // Check for else block
            if let Some(else_start) = alt_def.get("else_end_message") {
                if msg_idx == else_start.as_u64().unwrap_or(0) as usize {
                    let else_cond = alt_def
                        .get("else_condition")
                        .and_then(|v| v.as_str())
                        .unwrap_or("else");
                    mermaid.push_str(&format!("    else {}\n", else_cond));
                }
            }
        }

        // Add notes before message if specified
        for note in &notes {
            if let Some(after_idx) = note.get("after_message").and_then(|v| v.as_u64()) {
                if msg_idx == after_idx as usize {
                    mermaid.push_str(&format_note(note)?);
                }
            }
        }

        // Generate message
        let from = message
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Message missing 'from' field"))?;
        let to = message
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Message missing 'to' field"))?;
        let text = message
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Message missing 'text' field"))?;
        let msg_type = message
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("solid");

        let activate = message
            .get("activate")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let deactivate = message
            .get("deactivate")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let arrow = match msg_type {
            "solid" => "->",
            "dotted" => "-->",
            "solid_open" => "->>",
            "dotted_open" => "-->>",
            _ => anyhow::bail!("Invalid message type: {}", msg_type),
        };

        let indent = if current_loop.is_some() || current_alt.is_some() {
            "        "
        } else {
            "    "
        };

        if activate {
            mermaid.push_str(&format!("{}{}->>+{}: {}\n", indent, from, to, text));
        } else if deactivate {
            mermaid.push_str(&format!("{}{}->>-{}: {}\n", indent, from, to, text));
        } else {
            mermaid.push_str(&format!("{}{}{}{}: {}\n", indent, from, arrow, to, text));
        }

        // Check if we need to end a loop
        if let Some(loop_idx) = current_loop {
            let end = loops[loop_idx]
                .get("end_message")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Loop missing 'end_message'"))?
                as usize;

            if msg_idx == end {
                mermaid.push_str("    end\n");
                current_loop = None;
            }
        }

        // Check if we need to end an alt
        if let Some(alt_idx) = current_alt {
            let end = alts[alt_idx]
                .get("end_message")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Alt missing 'end_message'"))?
                as usize;

            if msg_idx == end {
                mermaid.push_str("    end\n");
                current_alt = None;
            }
        }
    }

    // Add standalone notes (no after_message specified)
    for note in &notes {
        if note.get("after_message").is_none() {
            mermaid.push_str(&format_note(note)?);
        }
    }

    Ok(mermaid)
}

/// Format a note
fn format_note(note: &Value) -> Result<String> {
    let text = note
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Note missing 'text' field"))?;
    let position = note
        .get("position")
        .and_then(|v| v.as_str())
        .unwrap_or("right_of");

    let note_str = match position {
        "right_of" => {
            let participant = note
                .get("participant")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Note missing 'participant' field"))?;
            format!("    Note right of {}: {}\n", participant, text)
        }
        "left_of" => {
            let participant = note
                .get("participant")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Note missing 'participant' field"))?;
            format!("    Note left of {}: {}\n", participant, text)
        }
        "over" => {
            let participants = note
                .get("participants")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Note 'over' position requires 'participants' array"))?;

            let participant_ids: Vec<&str> = participants
                .iter()
                .filter_map(|v| v.as_str())
                .collect();

            if participant_ids.is_empty() {
                anyhow::bail!("Note 'over' position requires at least one participant");
            }

            format!(
                "    Note over {}: {}\n",
                participant_ids.join(","),
                text
            )
        }
        _ => anyhow::bail!("Invalid note position: {}", position),
    };

    Ok(note_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sequence() {
        let args = json!({
            "participants": [
                {"id": "Client", "label": "Client", "type": "actor"},
                {"id": "Server", "label": "Server", "type": "participant"}
            ],
            "messages": [
                {"from": "Client", "to": "Server", "text": "Request", "type": "solid"},
                {"from": "Server", "to": "Client", "text": "Response", "type": "dotted"}
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("sequenceDiagram"));
        assert!(result.contains("actor Client as Client"));
        assert!(result.contains("participant Server as Server"));
        assert!(result.contains("Client->Server: Request"));
        assert!(result.contains("Server-->Client: Response"));
    }

    #[test]
    fn test_sequence_with_loop() {
        let args = json!({
            "participants": [
                {"id": "A", "label": "Service A"},
                {"id": "B", "label": "Service B"}
            ],
            "messages": [
                {"from": "A", "to": "B", "text": "Start"},
                {"from": "B", "to": "A", "text": "Processing"},
                {"from": "A", "to": "B", "text": "Continue"}
            ],
            "loops": [
                {
                    "label": "Every 5 seconds",
                    "start_message": 1,
                    "end_message": 2
                }
            ]
        });

        let result = execute(&args).unwrap();
        assert!(result.contains("loop Every 5 seconds"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_invalid_participants() {
        let args = json!({
            "participants": [
                {"id": "A", "label": "Only one"}
            ],
            "messages": [
                {"from": "A", "to": "B", "text": "Test"}
            ]
        });

        let result = execute(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("At least two participants are required"));
    }
}
