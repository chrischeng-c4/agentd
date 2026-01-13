use crate::Result;
use colored::Colorize;
use std::env;

pub struct ArchiveCommand;

pub async fn run(change_id: &str) -> Result<()> {
    let project_root = env::current_dir()?;

    println!("{}", format!("📦 Archiving: {}", change_id).cyan());

    let specter_dir = project_root.join("specter");
    let change_dir = specter_dir.join("changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", change_id);
    }

    // Create archive directory with timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d");
    let archive_name = format!("{}-{}", timestamp, change_id);
    let archive_dir = specter_dir.join("archive").join(&archive_name);

    std::fs::create_dir_all(&archive_dir)?;

    // Clean up dynamically generated context files
    crate::context::cleanup_context_files(&change_dir)?;

    // Move change directory to archive
    let archived_path = archive_dir.join(change_id);
    std::fs::rename(&change_dir, &archived_path)?;

    println!("\n{}", "✅ Archived successfully!".green().bold());
    println!("   Location: {}", archive_dir.display());

    Ok(())
}
