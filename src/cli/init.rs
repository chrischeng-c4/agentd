use crate::{Result, models::SpecterConfig};
use colored::Colorize;
use std::env;
use std::path::Path;

pub async fn run(name: Option<&str>) -> Result<()> {
    let project_root = env::current_dir()?;

    // Check if already initialized
    let specter_dir = project_root.join(".specter");
    if specter_dir.exists() {
        anyhow::bail!("Specter is already initialized in this directory");
    }

    println!("{}", "🚀 Initializing Specter...".cyan());

    // Create directory structure
    std::fs::create_dir_all(&specter_dir)?;
    std::fs::create_dir_all(project_root.join("specs"))?;
    std::fs::create_dir_all(project_root.join("changes"))?;
    std::fs::create_dir_all(specter_dir.join("scripts"))?;
    std::fs::create_dir_all(specter_dir.join("templates"))?;

    // Create config
    let mut config = SpecterConfig::default();
    if let Some(n) = name {
        config.project_name = n.to_string();
    } else {
        // Try to get project name from current directory
        if let Some(dir_name) = project_root.file_name() {
            config.project_name = dir_name.to_string_lossy().to_string();
        }
    }
    config.scripts_dir = specter_dir.join("scripts");
    config.save(&project_root)?;

    // Create example scripts (user needs to implement these)
    create_example_scripts(&specter_dir.join("scripts"))?;

    println!("{}", "✅ Specter initialized successfully!".green().bold());
    println!("\n{}", "📁 Directory structure created:".cyan());
    println!("   .specter/");
    println!("     ├── config.toml");
    println!("     └── scripts/");
    println!("   specs/");
    println!("   changes/");

    println!("\n{}", "⏭️  Next steps:".yellow());
    println!("   1. Implement AI integration scripts in .specter/scripts/");
    println!("   2. Create your first proposal:");
    println!("      specter proposal my-feature \"Add new feature\"");

    Ok(())
}

fn create_example_scripts(scripts_dir: &Path) -> Result<()> {
    // Create example script templates
    let gemini_proposal = r#"#!/bin/bash
# Gemini proposal generation script
# Usage: ./gemini-proposal.sh <change-id> <description>

CHANGE_ID="$1"
DESCRIPTION="$2"

echo "Generating proposal for: $CHANGE_ID"
echo "Description: $DESCRIPTION"

# TODO: Implement Gemini CLI integration
# Example:
# gemini /openspec:proposal "$CHANGE_ID" "$DESCRIPTION" --output-format stream-json

echo "ERROR: Script not implemented yet"
exit 1
"#;

    let codex_challenge = r#"#!/bin/bash
# Codex challenge script
# Usage: ./codex-challenge.sh <change-id>

CHANGE_ID="$1"

echo "Challenging proposal: $CHANGE_ID"

# TODO: Implement Codex CLI integration
# Example:
# codex challenge "$CHANGE_ID" --output challenges/$CHANGE_ID/CHALLENGE.md

echo "ERROR: Script not implemented yet"
exit 1
"#;

    std::fs::write(scripts_dir.join("gemini-proposal.sh"), gemini_proposal)?;
    std::fs::write(scripts_dir.join("codex-challenge.sh"), codex_challenge)?;
    std::fs::write(scripts_dir.join("gemini-reproposal.sh"), "#!/bin/bash\necho 'Not implemented'\nexit 1")?;
    std::fs::write(scripts_dir.join("claude-implement.sh"), "#!/bin/bash\necho 'Not implemented'\nexit 1")?;
    std::fs::write(scripts_dir.join("codex-verify.sh"), "#!/bin/bash\necho 'Not implemented'\nexit 1")?;

    // Make scripts executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in &["gemini-proposal.sh", "codex-challenge.sh", "gemini-reproposal.sh",
                       "claude-implement.sh", "codex-verify.sh"] {
            let path = scripts_dir.join(script);
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }
    }

    Ok(())
}
