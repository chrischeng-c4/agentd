//! Tasks CLI commands

use crate::services::tasks_service::{self, CreateTasksInput, FileActionData, TaskData};
use crate::Result;
use clap::Subcommand;
use std::env;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum TasksCommands {
    /// Create tasks file from JSON file
    Create {
        /// Change ID
        change_id: String,

        /// JSON file with tasks data
        #[arg(long)]
        json_file: PathBuf,
    },
}

pub fn run(cmd: TasksCommands) -> Result<()> {
    let project_root = env::current_dir()?;

    match cmd {
        TasksCommands::Create {
            change_id,
            json_file,
        } => {
            // Read and parse JSON file
            let json_content = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;

            // Extract tasks array from JSON
            let tasks: Vec<TaskData> = json
                .get("tasks")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing 'tasks' field"))?
                .iter()
                .filter_map(|t| {
                    let file = t.get("file")?;
                    Some(TaskData {
                        layer: t.get("layer")?.as_str()?.to_string(),
                        number: t.get("number")?.as_u64()? as u32,
                        title: t.get("title")?.as_str()?.to_string(),
                        file: FileActionData {
                            path: file.get("path")?.as_str()?.to_string(),
                            action: file.get("action")?.as_str()?.to_string(),
                        },
                        spec_ref: t.get("spec_ref")?.as_str()?.to_string(),
                        description: t.get("description")?.as_str()?.to_string(),
                        depends: t
                            .get("depends")
                            .and_then(|d| d.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                })
                .collect();

            // Create input struct
            let input = CreateTasksInput { change_id, tasks };

            // Create tasks
            let result = tasks_service::create_tasks(input, &project_root)?;
            println!("{}", result);
        }
    }

    Ok(())
}
