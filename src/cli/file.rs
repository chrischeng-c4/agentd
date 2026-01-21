//! File CLI commands

use crate::services::file_service;
use crate::Result;
use clap::Subcommand;
use std::env;

#[derive(Subcommand)]
pub enum FileCommands {
    /// Read a file from change directory
    Read {
        /// Change ID
        change_id: String,

        /// File to read: "proposal", "tasks", or spec name
        #[arg(default_value = "proposal")]
        file: String,
    },
}

pub fn run(cmd: FileCommands) -> Result<()> {
    let project_root = env::current_dir()?;

    match cmd {
        FileCommands::Read { change_id, file } => {
            let result = file_service::read_file(&change_id, &file, &project_root)?;
            println!("{}", result);
        }
    }
    Ok(())
}
