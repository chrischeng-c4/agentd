use crate::Result;
use crate::parser::markdown::extract_heading_section;
use colored::Colorize;
use std::env;
use std::fs;

pub fn run(archived: bool) -> Result<()> {
    let project_root = env::current_dir()?;

    println!("{}", "📋 Listing changes...".cyan());

    let agentd_dir = project_root.join("agentd");
    let changes_dir = agentd_dir.join("changes");
    if !changes_dir.exists() {
        println!("{}", "No changes found. Run 'agentd init' first.".yellow());
        return Ok(());
    }

    // List active changes
    if !archived {
        println!("\n{}", "Active changes:".green().bold());
        for entry in std::fs::read_dir(&changes_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    println!("   • {}", name.to_string_lossy());
                }
            }
        }
    } else {
        // List archived changes
        let archive_dir = agentd_dir.join("archive");
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

/// Represents an archived change with metadata
struct ArchivedChange {
    date: String,
    change_id: String,
    summary: String,
}

/// Lists archived changes with detailed information (date, ID, summary).
pub fn run_archived_detailed() -> Result<()> {
    let project_root = env::current_dir()?;
    run_archived_detailed_impl(&project_root)
}

/// Internal implementation that accepts project_root for testability.
/// This prevents tests from mutating the global CWD.
fn run_archived_detailed_impl(project_root: &std::path::Path) -> Result<()> {
    let agentd_dir = project_root.join("agentd");
    let archive_dir = agentd_dir.join("archive");

    // Check if archive directory exists
    if !archive_dir.exists() {
        println!("{}", "No archived changes found.".yellow());
        return Ok(());
    }

    // Collect archived changes
    let mut archived_changes: Vec<ArchivedChange> = Vec::new();

    for entry in fs::read_dir(&archive_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let folder_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Parse folder name: {YYYYMMDD}-{change_id}
        if let Some((date_str, change_id)) = parse_archive_folder_name(&folder_name) {
            // Format date from YYYYMMDD to YYYY-MM-DD
            let formatted_date = format_date(&date_str);

            // Read proposal.md and extract summary
            let proposal_path = path.join(&change_id).join("proposal.md");
            let summary = if proposal_path.exists() {
                let content = fs::read_to_string(&proposal_path)?;
                extract_heading_section(&content, "Summary")
            } else {
                String::new()
            };

            archived_changes.push(ArchivedChange {
                date: formatted_date,
                change_id: change_id.to_string(),
                summary,
            });
        } else {
            eprintln!("{}", format!("⚠️  Skipping malformed folder: {}", folder_name).yellow());
        }
    }

    // Check if we have any archived changes
    if archived_changes.is_empty() {
        println!("{}", "No archived changes found.".yellow());
        return Ok(());
    }

    // Sort by date (newest first)
    archived_changes.sort_by(|a, b| b.date.cmp(&a.date));

    // Display the table
    println!("\n{}", "Archived changes:".green().bold());
    println!();

    // Print header
    println!("{:<12} {:<30} {}", "Date".bold(), "ID".bold(), "Summary".bold());
    println!("{}", "─".repeat(100));

    // Print rows
    for change in archived_changes {
        let summary_display = if change.summary.is_empty() {
            "(no summary)".dimmed().to_string()
        } else {
            change.summary
        };
        println!("{:<12} {:<30} {}", change.date, change.change_id, summary_display);
    }

    println!();

    Ok(())
}

/// Parses an archive folder name in the format {YYYYMMDD}-{change_id}.
/// Returns Some((date_str, change_id)) if valid, None otherwise.
fn parse_archive_folder_name(folder_name: &str) -> Option<(String, String)> {
    // Split on first hyphen
    let parts: Vec<&str> = folder_name.splitn(2, '-').collect();

    if parts.len() != 2 {
        return None;
    }

    let date_str = parts[0];
    let change_id = parts[1];

    // Validate date format (YYYYMMDD - exactly 8 digits)
    if date_str.len() != 8 || !date_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    // Validate change_id is non-empty
    if change_id.is_empty() {
        return None;
    }

    Some((date_str.to_string(), change_id.to_string()))
}

/// Formats a date string from YYYYMMDD to YYYY-MM-DD.
fn format_date(date_str: &str) -> String {
    if date_str.len() == 8 {
        format!("{}-{}-{}", &date_str[0..4], &date_str[4..6], &date_str[6..8])
    } else {
        date_str.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_archive_folder_name_valid() {
        let result = parse_archive_folder_name("20260116-test-retry");
        assert_eq!(result, Some(("20260116".to_string(), "test-retry".to_string())));
    }

    #[test]
    fn test_parse_archive_folder_name_with_multiple_hyphens() {
        let result = parse_archive_folder_name("20260116-test-retry-feature");
        assert_eq!(result, Some(("20260116".to_string(), "test-retry-feature".to_string())));
    }

    #[test]
    fn test_parse_archive_folder_name_invalid_date_length() {
        let result = parse_archive_folder_name("2026011-test");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_archive_folder_name_invalid_date_format() {
        let result = parse_archive_folder_name("2026011a-test");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_archive_folder_name_no_hyphen() {
        let result = parse_archive_folder_name("20260116test");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_archive_folder_name_empty_change_id() {
        let result = parse_archive_folder_name("20260116-");
        assert_eq!(result, None);
    }

    #[test]
    fn test_format_date() {
        let result = format_date("20260116");
        assert_eq!(result, "2026-01-16");
    }

    #[test]
    fn test_format_date_invalid() {
        let result = format_date("2026");
        assert_eq!(result, "2026");
    }

    #[test]
    fn test_run_archived_detailed_empty_archive() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let agentd_dir = temp_dir.path().join("agentd");
        let archive_dir = agentd_dir.join("archive");
        fs::create_dir_all(&archive_dir).unwrap();

        // Run the command with explicit project root - no CWD mutation
        let result = run_archived_detailed_impl(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_archived_detailed_malformed_folders() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let agentd_dir = temp_dir.path().join("agentd");
        let archive_dir = agentd_dir.join("archive");
        fs::create_dir_all(&archive_dir).unwrap();

        // Create malformed folders (should be skipped)
        fs::create_dir_all(archive_dir.join("invalid-folder")).unwrap();
        fs::create_dir_all(archive_dir.join("2026011-short")).unwrap();
        fs::create_dir_all(archive_dir.join("no-date-prefix")).unwrap();
        fs::create_dir_all(archive_dir.join("20260116-")).unwrap(); // Empty change ID

        // Run the command - should succeed and skip malformed folders
        // Note: The function prints warnings to stderr for malformed folders
        let result = run_archived_detailed_impl(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_archived_detailed_missing_proposal() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let agentd_dir = temp_dir.path().join("agentd");
        let archive_dir = agentd_dir.join("archive");
        fs::create_dir_all(&archive_dir).unwrap();

        // Create a valid archive folder but without proposal.md
        let change_folder = archive_dir.join("20260116-test-change");
        let change_dir = change_folder.join("test-change");
        fs::create_dir_all(&change_dir).unwrap();

        // Run the command - should succeed with empty summary
        let result = run_archived_detailed_impl(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_archived_detailed_with_valid_proposal() {
        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let agentd_dir = temp_dir.path().join("agentd");
        let archive_dir = agentd_dir.join("archive");
        fs::create_dir_all(&archive_dir).unwrap();

        // Create a valid archive folder with proposal.md
        let change_folder = archive_dir.join("20260116-test-change");
        let change_dir = change_folder.join("test-change");
        fs::create_dir_all(&change_dir).unwrap();

        let proposal_path = change_dir.join("proposal.md");
        let mut file = fs::File::create(&proposal_path).unwrap();
        writeln!(file, "## Summary\n\nThis is a test summary.").unwrap();

        // Run the command - should succeed and extract summary
        let result = run_archived_detailed_impl(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_archived_detailed_no_archive_dir() {
        // Create a temporary directory structure without archive dir
        let temp_dir = TempDir::new().unwrap();
        let agentd_dir = temp_dir.path().join("agentd");
        fs::create_dir_all(&agentd_dir).unwrap();

        // Run the command - should succeed with "no archived changes" message
        let result = run_archived_detailed_impl(temp_dir.path());
        assert!(result.is_ok());
    }
}
