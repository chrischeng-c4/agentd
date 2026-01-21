//! Implementation CLI commands

use crate::services::implementation_service;
use crate::Result;
use clap::Subcommand;
use std::env;

#[derive(Subcommand)]
pub enum ImplementationCommands {
    /// Read all requirements (proposal, tasks, specs) in one call
    ReadAll {
        /// Change ID
        change_id: String,
    },

    /// List changed files with statistics
    ListFiles {
        /// Change ID
        change_id: String,

        /// Base branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base_branch: String,

        /// Optional filter pattern (simple string match)
        #[arg(long)]
        filter: Option<String>,
    },
}

pub fn run(cmd: ImplementationCommands) -> Result<()> {
    let project_root = env::current_dir()?;

    match cmd {
        ImplementationCommands::ReadAll { change_id } => {
            let result = implementation_service::read_all_requirements(&change_id, &project_root)?;
            println!("{}", result);
        }
        ImplementationCommands::ListFiles {
            change_id,
            base_branch,
            filter,
        } => {
            let result = implementation_service::list_changed_files(
                &change_id,
                Some(&base_branch),
                filter.as_deref(),
                &project_root,
            )?;
            println!("{}", result);
        }
    }
    Ok(())
}
