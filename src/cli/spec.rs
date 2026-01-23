//! Spec CLI commands

use crate::services::file_service;
use crate::services::spec_service::{self, CreateSpecInput, RequirementData, ScenarioData};
use crate::Result;
use clap::Subcommand;
use std::env;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum SpecCommands {
    /// List all spec files in a change
    List {
        /// Change ID
        change_id: String,
    },

    /// Create a new spec from JSON file
    Create {
        /// Change ID
        change_id: String,

        /// Spec ID
        spec_id: String,

        /// JSON file with spec data
        #[arg(long)]
        json_file: PathBuf,
    },
}

pub fn run(cmd: SpecCommands) -> Result<()> {
    let project_root = env::current_dir()?;

    match cmd {
        SpecCommands::List { change_id } => {
            let result = file_service::list_specs(&change_id, None, &project_root)?;
            println!("{}", result);
        }

        SpecCommands::Create {
            change_id,
            spec_id,
            json_file,
        } => {
            // Read and parse JSON file
            let json_content = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;

            // Extract fields from JSON
            let title = json
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'title' field"))?
                .to_string();

            let overview = json
                .get("overview")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'overview' field"))?
                .to_string();

            let requirements: Vec<RequirementData> = json
                .get("requirements")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing 'requirements' field"))?
                .iter()
                .filter_map(|r| {
                    Some(RequirementData {
                        id: r.get("id")?.as_str()?.to_string(),
                        title: r.get("title")?.as_str()?.to_string(),
                        description: r.get("description")?.as_str()?.to_string(),
                        priority: r
                            .get("priority")
                            .and_then(|p| p.as_str())
                            .unwrap_or("medium")
                            .to_string(),
                    })
                })
                .collect();

            let scenarios: Vec<ScenarioData> = json
                .get("scenarios")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing 'scenarios' field"))?
                .iter()
                .filter_map(|s| {
                    Some(ScenarioData {
                        name: s.get("name")?.as_str()?.to_string(),
                        given: s.get("given").and_then(|g| g.as_str()).map(String::from),
                        when: s.get("when")?.as_str()?.to_string(),
                        then: s.get("then")?.as_str()?.to_string(),
                    })
                })
                .collect();

            let flow_diagram = json
                .get("flow_diagram")
                .and_then(|v| v.as_str())
                .map(String::from);

            let data_model = json.get("data_model").cloned();

            // Create input struct
            let input = CreateSpecInput {
                change_id,
                spec_id,
                title,
                overview,
                requirements,
                scenarios,
                flow_diagram,
                data_model,
            };

            // Create spec
            let result = spec_service::create_spec(input, &project_root)?;
            println!("{}", result);
        }
    }
    Ok(())
}
