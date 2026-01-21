//! Clarifications CLI command

use crate::services::clarifications_service::{
    self, CreateClarificationsInput, QuestionAnswer,
};
use crate::Result;
use clap::Args;
use std::env;
use std::path::PathBuf;

#[derive(Args)]
pub struct ClarificationsArgs {
    /// Change ID
    change_id: String,

    /// JSON file with clarifications data
    #[arg(long)]
    json_file: PathBuf,
}

pub fn run(args: ClarificationsArgs) -> Result<()> {
    let project_root = env::current_dir()?;

    // Read JSON file
    let json_str = std::fs::read_to_string(&args.json_file)?;
    let json: serde_json::Value = serde_json::from_str(&json_str)?;

    // Extract questions array
    let questions: Vec<QuestionAnswer> = json
        .get("questions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing 'questions' field"))?
        .iter()
        .filter_map(|q| {
            Some(QuestionAnswer {
                topic: q.get("topic")?.as_str()?.to_string(),
                question: q.get("question")?.as_str()?.to_string(),
                answer: q.get("answer")?.as_str()?.to_string(),
                rationale: q.get("rationale")?.as_str()?.to_string(),
            })
        })
        .collect();

    let input = CreateClarificationsInput {
        change_id: args.change_id,
        questions,
    };

    let result = clarifications_service::create_clarifications(input, &project_root)?;
    println!("{}", result);
    Ok(())
}
