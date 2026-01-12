use crate::Result;
use colored::Colorize;
use std::env;

pub async fn run(archived: bool) -> Result<()> {
    let project_root = env::current_dir()?;

    println!("{}", "📋 Listing changes...".cyan());

    let changes_dir = project_root.join("changes");
    if !changes_dir.exists() {
        println!("{}", "No changes found. Run 'specter init' first.".yellow());
        return Ok(());
    }

    // List active changes
    if !archived {
        println!("\n{}", "Active changes:".green().bold());
        for entry in std::fs::read_dir(&changes_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap() != "archive" {
                if let Some(name) = path.file_name() {
                    println!("   • {}", name.to_string_lossy());
                }
            }
        }
    } else {
        // List archived changes
        let archive_dir = changes_dir.join("archive");
        if archive_dir.exists() {
            println!("\n{}", "Archived changes:".green().bold());
            for entry in std::fs::read_dir(&archive_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(name) = entry.path().file_name() {
                        println!("   • {}", name.to_string_lossy());
                    }
                }
            }
        } else {
            println!("{}", "No archived changes found.".yellow());
        }
    }

    Ok(())
}
