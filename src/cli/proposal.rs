//! Proposal CLI commands

use crate::services::proposal_service::{self, AffectedSpec, CreateProposalInput, ImpactData};
use crate::Result;
use clap::Subcommand;
use std::env;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ProposalCommands {
    /// Create a new proposal from JSON file
    Create {
        /// Change ID
        change_id: String,

        /// JSON file with proposal data
        #[arg(long)]
        json_file: PathBuf,
    },

    /// Add review to proposal from JSON file
    Review {
        /// Change ID
        change_id: String,

        /// JSON file with review data
        #[arg(long)]
        json_file: PathBuf,
    },
}

pub fn run(cmd: ProposalCommands) -> Result<()> {
    let project_root = env::current_dir()?;

    match cmd {
        ProposalCommands::Create {
            change_id,
            json_file,
        } => {
            // Read and parse JSON file
            let json_content = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;

            // Extract fields from JSON
            let summary = json
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'summary' field"))?
                .to_string();

            let why = json
                .get("why")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'why' field"))?
                .to_string();

            let what_changes: Vec<String> = json
                .get("what_changes")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing 'what_changes' field"))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            let impact = json
                .get("impact")
                .ok_or_else(|| anyhow::anyhow!("Missing 'impact' field"))?;

            let scope = impact
                .get("scope")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'impact.scope' field"))?
                .to_string();

            let affected_files = impact
                .get("affected_files")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'impact.affected_files' field"))?;

            let new_files = impact.get("new_files").and_then(|v| v.as_i64()).unwrap_or(0);

            let affected_specs: Vec<AffectedSpec> = impact
                .get("affected_specs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            // Support both old format (string) and new format (object)
                            if let Some(s) = v.as_str() {
                                // Old format: just a string ID
                                Some(AffectedSpec {
                                    id: s.to_string(),
                                    depends: vec![],
                                })
                            } else if let Some(obj) = v.as_object() {
                                // New format: object with id and depends
                                let id = obj.get("id")?.as_str()?.to_string();
                                let depends = obj
                                    .get("depends")
                                    .and_then(|d| d.as_array())
                                    .map(|deps| {
                                        deps.iter()
                                            .filter_map(|d| d.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                Some(AffectedSpec { id, depends })
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let affected_code: Vec<String> = impact
                .get("affected_code")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let breaking_changes = impact
                .get("breaking_changes")
                .and_then(|v| v.as_str())
                .map(String::from);

            // Create input struct
            let input = CreateProposalInput {
                change_id,
                summary,
                why,
                what_changes,
                impact: ImpactData {
                    scope,
                    affected_files,
                    new_files,
                    affected_specs,
                    affected_code,
                    breaking_changes,
                },
            };

            // Create proposal
            let result = proposal_service::create_proposal(input, &project_root)?;
            println!("{}", result);
        }

        ProposalCommands::Review {
            change_id,
            json_file,
        } => {
            // Read and parse JSON file
            let json_content = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;

            // Extract fields from JSON
            let status = json
                .get("status")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'status' field"))?;

            let iteration = json
                .get("iteration")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'iteration' field"))? as u32;

            let reviewer = json
                .get("reviewer")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'reviewer' field"))?;

            let content = json
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'content' field"))?;

            // Get proposal path
            let proposal_path = project_root
                .join("agentd/changes")
                .join(&change_id)
                .join("proposal.md");

            if !proposal_path.exists() {
                anyhow::bail!(
                    "proposal.md not found for change '{}'. Run create proposal first.",
                    change_id
                );
            }

            // Append review
            proposal_service::append_review(&proposal_path, status, iteration, reviewer, content)?;

            println!(
                "Appended review block (status={}, iteration={}) to proposal.md for change '{}'",
                status, iteration, change_id
            );
        }
    }

    Ok(())
}
