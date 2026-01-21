use crate::Result;
use colored::Colorize;
use std::path::Path;

/// Migrate old format files to XML format
pub async fn run(change_id: Option<&str>) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let changes_dir = project_root.join("agentd/changes");

    if !changes_dir.exists() {
        anyhow::bail!("agentd/changes directory not found");
    }

    if let Some(id) = change_id {
        // Migrate specific change
        let change_dir = changes_dir.join(id);
        if !change_dir.exists() {
            anyhow::bail!("Change '{}' not found", id);
        }
        migrate_change(&change_dir)?;
    } else {
        // Migrate all changes
        println!("{}", "🔄 Migrating all changes to XML format...".cyan());
        let mut count = 0;

        for entry in std::fs::read_dir(&changes_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Err(e) = migrate_change(&entry.path()) {
                    eprintln!(
                        "   {} Failed to migrate {}: {}",
                        "⚠".yellow(),
                        entry.path().display(),
                        e
                    );
                } else {
                    count += 1;
                }
            }
        }

        println!("\n{}", format!("✓ Migrated {} changes", count).green());
    }

    Ok(())
}

fn migrate_change(change_dir: &Path) -> Result<()> {
    let change_id = change_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid change directory"))?;

    println!("\n{}", format!("Migrating: {}", change_id).cyan());

    // Migrate proposal.md
    let proposal_path = change_dir.join("proposal.md");
    if proposal_path.exists() {
        migrate_file(&proposal_path, "proposal")?;
    } else {
        println!("   {} proposal.md not found", "⚠".yellow());
    }

    // Migrate CHALLENGE.md to review block in proposal.md
    let challenge_path = change_dir.join("CHALLENGE.md");
    if challenge_path.exists() && proposal_path.exists() {
        migrate_challenge_to_review(&proposal_path, &challenge_path)?;
    }

    // Migrate specs
    let specs_dir = change_dir.join("specs");
    if specs_dir.exists() {
        for entry in std::fs::read_dir(&specs_dir)? {
            let path = entry?.path();
            if path.extension() == Some(std::ffi::OsStr::new("md")) {
                migrate_file(&path, "spec")?;
            }
        }
    }

    // Migrate tasks.md
    let tasks_path = change_dir.join("tasks.md");
    if tasks_path.exists() {
        migrate_file(&tasks_path, "tasks")?;
    }

    println!("  {} Migration complete", "✓".green());
    Ok(())
}

fn migrate_file(path: &Path, tag: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;

    // Check if already migrated
    if content.contains(&format!("<{}>", tag)) {
        println!(
            "   {} {}: already in XML format",
            "→".bright_black(),
            path.file_name().unwrap().to_string_lossy()
        );
        return Ok(());
    }

    // Split frontmatter and body
    let (frontmatter, body) = split_frontmatter(&content)?;

    // Wrap body in XML
    let migrated = format!(
        "---\n{}\n---\n\n<{}>\n{}\n</{}>\n",
        frontmatter.trim_matches('-').trim(),
        tag,
        body.trim(),
        tag
    );

    std::fs::write(path, migrated)?;
    println!(
        "   {} {}: wrapped in <{}> tag",
        "✓".green(),
        path.file_name().unwrap().to_string_lossy(),
        tag
    );
    Ok(())
}

fn migrate_challenge_to_review(proposal_path: &Path, challenge_path: &Path) -> Result<()> {
    let challenge_content = std::fs::read_to_string(challenge_path)?;

    // Parse verdict using the existing parser
    let verdict = crate::parser::parse_challenge_verdict(challenge_path)?;
    let status = match verdict {
        crate::models::ChallengeVerdict::Approved => "approved",
        crate::models::ChallengeVerdict::NeedsRevision => "needs_revision",
        crate::models::ChallengeVerdict::Rejected => "rejected",
        _ => "unknown",
    };

    // Append as review block using the service layer
    crate::services::proposal_service::append_review(proposal_path, status, 1, "codex", &challenge_content)?;

    println!(
        "   {} CHALLENGE.md: migrated to <review> block",
        "✓".green()
    );

    // Optionally rename CHALLENGE.md to CHALLENGE.md.old
    let old_path = challenge_path.with_extension("md.old");
    std::fs::rename(challenge_path, &old_path)?;
    println!(
        "   {} CHALLENGE.md: renamed to CHALLENGE.md.old",
        "→".bright_black()
    );

    Ok(())
}

/// Split frontmatter from body
fn split_frontmatter(content: &str) -> Result<(String, String)> {
    if !content.starts_with("---") {
        return Ok((String::new(), content.to_string()));
    }

    let remaining = &content[3..];
    if let Some(end_pos) = remaining.find("\n---") {
        let frontmatter = &remaining[..end_pos];
        let body = &remaining[end_pos + 4..];
        Ok((frontmatter.to_string(), body.to_string()))
    } else {
        Ok((String::new(), content.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter() {
        let content = r#"---
id: test
type: proposal
---

# Content here
"#;

        let (frontmatter, body) = split_frontmatter(content).unwrap();
        assert!(frontmatter.contains("id: test"));
        assert!(body.contains("# Content here"));
    }

    #[test]
    fn test_split_frontmatter_no_frontmatter() {
        let content = "# Just content";
        let (frontmatter, body) = split_frontmatter(content).unwrap();
        assert!(frontmatter.is_empty());
        assert_eq!(body, content);
    }
}
