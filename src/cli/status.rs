use crate::Result;
use colored::Colorize;

pub async fn run(change_id: &str, json: bool) -> Result<()> {
    if json {
        println!("{{\"status\": \"not_implemented\"}}");
    } else {
        println!("{}", format!("Status for: {}", change_id).cyan());
        println!("{}", "🚧 Not implemented yet".yellow());
    }
    Ok(())
}
