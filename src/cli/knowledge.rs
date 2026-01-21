//! Knowledge CLI commands

use crate::services::knowledge_service::{self, WriteKnowledgeInput};
use crate::Result;
use clap::Subcommand;
use std::env;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum KnowledgeCommands {
    /// Read a knowledge document
    Read {
        /// Path relative to agentd/knowledge/ (e.g., "00-architecture/index.md")
        path: String,
    },

    /// List all knowledge documents
    List {
        /// Optional subdirectory to filter (e.g., "00-architecture")
        path: Option<String>,
    },

    /// Write or update a knowledge document
    Write {
        /// Path relative to agentd/knowledge/ (e.g., "30-claude/skills.md")
        path: String,

        /// JSON file with knowledge data (title, source, content)
        #[arg(long)]
        json_file: PathBuf,
    },

    /// Write or update a spec in agentd/specs/ (for archive merge)
    WriteSpec {
        /// Path relative to agentd/specs/ (e.g., "math-utility.md")
        path: String,

        /// File containing the full spec content (including frontmatter)
        #[arg(long)]
        content_file: PathBuf,
    },
}

pub fn run(cmd: KnowledgeCommands) -> Result<()> {
    let project_root = env::current_dir()?;

    match cmd {
        KnowledgeCommands::Read { path } => {
            let result = knowledge_service::read_knowledge(&path, &project_root)?;
            println!("{}", result);
        }
        KnowledgeCommands::List { path } => {
            let result = knowledge_service::list_knowledge(path.as_deref(), &project_root)?;
            println!("{}", result);
        }
        KnowledgeCommands::Write { path, json_file } => {
            // Read and parse JSON file
            let json_str = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_str)?;

            let title = json
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'title' field"))?
                .to_string();

            let source = json
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'source' field"))?
                .to_string();

            let content = json
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'content' field"))?
                .to_string();

            let input = WriteKnowledgeInput {
                path,
                title,
                source,
                content,
            };

            let result = knowledge_service::write_knowledge(input, &project_root)?;
            println!("{}", result);
        }
        KnowledgeCommands::WriteSpec { path, content_file } => {
            let content = std::fs::read_to_string(&content_file)?;
            let result = knowledge_service::write_main_spec(&path, &content, &project_root)?;
            println!("{}", result);
        }
    }
    Ok(())
}
