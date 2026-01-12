use crate::{Result, models::SpecterConfig};
use colored::Colorize;
use std::env;
use std::path::Path;

const SKILL_PROPOSAL: &str = include_str!("../../templates/skills/specter-proposal/SKILL.md");
const SKILL_CHALLENGE: &str = include_str!("../../templates/skills/specter-challenge/SKILL.md");

pub async fn run(name: Option<&str>) -> Result<()> {
    let project_root = env::current_dir()?;

    // Check if already initialized
    let specter_dir = project_root.join(".specter");
    let claude_dir = project_root.join(".claude");

    if specter_dir.exists() {
        println!("{}", "⚠️  Specter is already initialized".yellow());
        println!("   Run with --force to reinstall");
        return Ok(());
    }

    println!("{}", "🎭 Initializing Specter for Claude Code...".cyan().bold());
    println!();

    // Create directory structure
    println!("{}", "📁 Creating directory structure...".cyan());
    std::fs::create_dir_all(&specter_dir)?;
    std::fs::create_dir_all(project_root.join("specs"))?;
    std::fs::create_dir_all(project_root.join("changes"))?;
    std::fs::create_dir_all(specter_dir.join("scripts"))?;

    // Create Claude Code skills directory
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Create config
    let mut config = SpecterConfig::default();
    if let Some(n) = name {
        config.project_name = n.to_string();
    } else {
        if let Some(dir_name) = project_root.file_name() {
            config.project_name = dir_name.to_string_lossy().to_string();
        }
    }
    config.scripts_dir = specter_dir.join("scripts");
    config.save(&project_root)?;

    // Install Claude Code Skills
    println!("{}", "🤖 Installing Claude Code Skills...".cyan());
    install_claude_skills(&skills_dir)?;

    // Create helper scripts
    create_helper_scripts(&specter_dir.join("scripts"))?;

    println!();
    println!("{}", "✅ Specter initialized successfully!".green().bold());
    println!();
    println!("{}", "📁 Structure:".cyan());
    println!("   .specter/           - Configuration");
    println!("   .claude/skills/     - 6 Skills installed");
    println!("   specs/              - Main specifications");
    println!("   changes/            - Active changes");
    println!();

    println!("{}", "🎯 Available Skills (use in Claude Code):".cyan().bold());
    println!("   {} - Generate proposal with Gemini", "/specter:proposal".green());
    println!("   {} - Challenge proposal with Codex", "/specter:challenge".green());
    println!("   {} - Refine based on feedback", "/specter:reproposal".green());
    println!("   {} - Implement with Claude", "/specter:implement".green());
    println!("   {} - Verify with tests", "/specter:verify".green());
    println!("   {} - Archive completed change", "/specter:archive".green());
    println!();

    println!("{}", "⏭️  Next Steps:".yellow().bold());
    println!("   1. In Claude Code, run:");
    println!("      {}", "/specter:proposal my-feature \"Add awesome feature\"".cyan());
    println!();
    println!("   2. Configure API keys (optional):");
    println!("      Edit .specter/scripts/config.sh");
    println!();
    println!("   3. Read the guide:");
    println!("      cat .specter/README.md");

    Ok(())
}

fn install_claude_skills(skills_dir: &Path) -> Result<()> {
    // Install proposal skill
    let proposal_dir = skills_dir.join("specter-proposal");
    std::fs::create_dir_all(&proposal_dir)?;
    std::fs::write(proposal_dir.join("SKILL.md"), SKILL_PROPOSAL)?;
    println!("   ✓ specter-proposal");

    // Install challenge skill
    let challenge_dir = skills_dir.join("specter-challenge");
    std::fs::create_dir_all(&challenge_dir)?;
    std::fs::write(challenge_dir.join("SKILL.md"), SKILL_CHALLENGE)?;
    println!("   ✓ specter-challenge");

    // Install other skills (placeholders for now)
    for skill in &["reproposal", "implement", "verify", "archive"] {
        let skill_dir = skills_dir.join(format!("specter-{}", skill));
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(
            skill_dir.join("SKILL.md"),
            generate_placeholder_skill(skill)
        )?;
        println!("   ✓ specter-{}", skill);
    }

    Ok(())
}

fn generate_placeholder_skill(name: &str) -> String {
    format!(r#"---
name: specter-{}
description: {} (Coming soon)
user-invocable: true
---

# /specter:{} - {}

This skill is under development.

## Usage

```
/specter:{} <change-id>
```

## Placeholder

This skill will be implemented in the next version.
For now, you can use the specter CLI tool directly:

```bash
specter {} <change-id>
```
"#,
        name,
        name.replace('-', " ").to_uppercase(),
        name,
        name.replace('-', " ").to_uppercase(),
        name,
        name
    )
}

fn create_helper_scripts(scripts_dir: &Path) -> Result<()> {
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
