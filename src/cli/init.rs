use crate::{models::SpecterConfig, Result};
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};

// Claude Code Skills
const SKILL_PROPOSAL: &str = include_str!("../../templates/skills/specter-proposal/SKILL.md");
const SKILL_CHALLENGE: &str = include_str!("../../templates/skills/specter-challenge/SKILL.md");
const SKILL_REPROPOSAL: &str = include_str!("../../templates/skills/specter-reproposal/SKILL.md");
const SKILL_IMPLEMENT: &str = include_str!("../../templates/skills/specter-implement/SKILL.md");
const SKILL_VERIFY: &str = include_str!("../../templates/skills/specter-verify/SKILL.md");
const SKILL_FIX: &str = include_str!("../../templates/skills/specter-fix/SKILL.md");
const SKILL_ARCHIVE: &str = include_str!("../../templates/skills/specter-archive/SKILL.md");

// Gemini Commands
const GEMINI_PROPOSAL: &str = include_str!("../../templates/gemini/commands/specter/proposal.toml");
const GEMINI_REPROPOSAL: &str =
    include_str!("../../templates/gemini/commands/specter/reproposal.toml");
const GEMINI_SETTINGS: &str = include_str!("../../templates/gemini/settings.json");

// Codex Prompts
const CODEX_CHALLENGE: &str = include_str!("../../templates/codex/prompts/specter-challenge.md");
const CODEX_VERIFY: &str = include_str!("../../templates/codex/prompts/specter-verify.md");

// AI Context Files (GEMINI.md and AGENTS.md) are now generated dynamically per change
// from templates/GEMINI.md and templates/AGENTS.md

pub async fn run(name: Option<&str>) -> Result<()> {
    let project_root = env::current_dir()?;

    // Check if already initialized
    let specter_dir = project_root.join("specter");
    let claude_dir = project_root.join(".claude");

    if specter_dir.exists() {
        println!("{}", "⚠️  Specter is already initialized".yellow());
        println!("   Run with --force to reinstall");
        return Ok(());
    }

    println!(
        "{}",
        "🎭 Initializing Specter for Claude Code...".cyan().bold()
    );
    println!();

    // Create directory structure
    println!("{}", "📁 Creating directory structure...".cyan());
    std::fs::create_dir_all(&specter_dir)?;
    std::fs::create_dir_all(specter_dir.join("specs"))?;
    std::fs::create_dir_all(specter_dir.join("changes"))?;
    std::fs::create_dir_all(specter_dir.join("archive"))?;
    std::fs::create_dir_all(specter_dir.join("scripts"))?;

    // AI context files (GEMINI.md, AGENTS.md) are now generated dynamically
    // per change in specter/changes/<change-id>/ by the CLI commands

    // Create Claude Code skills directory
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Create config
    let mut config = SpecterConfig::default();
    if let Some(n) = name {
        config.project_name = n.to_string();
    } else if let Some(dir_name) = project_root.file_name() {
        config.project_name = dir_name.to_string_lossy().to_string();
    }
    config.scripts_dir = specter_dir.join("scripts");
    config.save(&project_root)?;

    // Install Claude Code Skills
    println!("{}", "🤖 Installing Claude Code Skills...".cyan());
    install_claude_skills(&skills_dir)?;

    // Install Gemini Commands (project-specific)
    println!();
    println!("{}", "🤖 Installing Gemini Commands...".cyan());
    let gemini_dir = claude_dir.parent().unwrap().join(".gemini");
    std::fs::create_dir_all(gemini_dir.join("commands/specter"))?;
    install_gemini_commands(&gemini_dir)?;

    // Install Codex Prompts (user-space)
    println!();
    println!("{}", "🤖 Installing Codex Prompts...".cyan());
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let codex_prompts_dir = PathBuf::from(home_dir).join(".codex/prompts");
    install_codex_prompts(&codex_prompts_dir)?;

    // Create helper scripts
    create_helper_scripts(&specter_dir.join("scripts"))?;

    println!();
    println!("{}", "✅ Specter initialized successfully!".green().bold());
    println!();
    println!("{}", "📁 Structure:".cyan());
    println!("   specter/                   - Main Specter directory");
    println!("   specter/specs/             - Main specifications");
    println!("   specter/changes/           - Active changes");
    println!("   specter/archive/           - Completed changes");
    println!("   .claude/skills/            - 7 Skills installed");
    println!("   .gemini/commands/specter/  - 2 Gemini commands");
    println!("   ~/.codex/prompts/          - 2 Codex prompts");
    println!();

    println!("{}", "🤖 AI Commands Installed:".cyan().bold());
    println!(
        "   {} - Proposal generation",
        "gemini specter:proposal".green()
    );
    println!(
        "   {} - Proposal refinement",
        "gemini specter:reproposal".green()
    );
    println!("   {} - Code review", "codex specter-challenge".green());
    println!("   {} - Test generation", "codex specter-verify".green());
    println!();

    println!(
        "{}",
        "🎯 Available Skills (use in Claude Code):".cyan().bold()
    );
    println!(
        "   {} - Generate proposal with Gemini",
        "/specter:proposal".green()
    );
    println!(
        "   {} - Challenge proposal with Codex",
        "/specter:challenge".green()
    );
    println!(
        "   {} - Refine based on feedback",
        "/specter:reproposal".green()
    );
    println!(
        "   {} - Implement with Claude",
        "/specter:implement".green()
    );
    println!("   {} - Verify with tests", "/specter:verify".green());
    println!("   {} - Fix verification failures", "/specter:fix".green());
    println!(
        "   {} - Archive completed change",
        "/specter:archive".green()
    );
    println!();

    println!("{}", "⏭️  Next Steps:".yellow().bold());
    println!("   1. In Claude Code, run:");
    println!(
        "      {}",
        "/specter:proposal my-feature \"Add awesome feature\"".cyan()
    );
    println!();
    println!("   2. Configure API keys (optional):");
    println!("      Edit specter/scripts/config.sh");
    println!();
    println!("   3. Read the guide:");
    println!("      cat specter/README.md");

    Ok(())
}

fn install_claude_skills(skills_dir: &Path) -> Result<()> {
    let skills = vec![
        ("proposal", SKILL_PROPOSAL),
        ("challenge", SKILL_CHALLENGE),
        ("reproposal", SKILL_REPROPOSAL),
        ("implement", SKILL_IMPLEMENT),
        ("verify", SKILL_VERIFY),
        ("fix", SKILL_FIX),
        ("archive", SKILL_ARCHIVE),
    ];

    for (name, content) in skills {
        let skill_dir = skills_dir.join(format!("specter-{}", name));
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(skill_dir.join("SKILL.md"), content)?;
        println!("   ✓ specter-{}", name);
    }

    Ok(())
}

fn create_helper_scripts(scripts_dir: &Path) -> Result<()> {
    // Create updated script templates that use AI CLI commands
    let gemini_proposal = r#"#!/bin/bash
# Gemini proposal generation script
# Usage: ./gemini-proposal.sh <change-id> <description>
set -euo pipefail

CHANGE_ID="$1"
DESCRIPTION="$2"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🤖 Generating proposal with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by specter CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/specter/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## User Request
${DESCRIPTION}

## Instructions
Create proposal files in specter/changes/${CHANGE_ID}/.
EOF
)

# Call Gemini CLI with pre-defined command
echo "$CONTEXT" | gemini specter:proposal --output-format stream-json
"#;

    let gemini_reproposal = r#"#!/bin/bash
# Gemini reproposal script
# Usage: ./gemini-reproposal.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔄 Refining proposal with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by specter CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/specter/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## Instructions
Read specter/changes/${CHANGE_ID}/CHALLENGE.md and fix all HIGH and MEDIUM severity issues.
EOF
)

# Call Gemini CLI with pre-defined command
# Use --resume latest to reuse the proposal session (cached codebase context)
echo "$CONTEXT" | gemini specter:reproposal --resume latest --output-format stream-json
"#;

    let codex_challenge = r#"#!/bin/bash
# Codex challenge script
# Usage: ./codex-challenge.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Analyzing proposal with Codex: $CHANGE_ID"

# Use change-specific AGENTS.md context (generated dynamically by specter CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/specter/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Specter Challenge Task

A skeleton CHALLENGE.md has been created at specter/changes/${CHANGE_ID}/CHALLENGE.md.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read all proposal files in specter/changes/${CHANGE_ID}/:
   - proposal.md
   - tasks.md
   - diagrams.md
   - specs/*.md

3. Explore the existing codebase

4. Fill the CHALLENGE.md skeleton with your findings:
   - **Internal Consistency Issues** (HIGH): Check if proposal docs match each other
   - **Code Alignment Issues** (MEDIUM/LOW): Check alignment with existing code
     - If proposal mentions "refactor" or "BREAKING", note deviations as intentional
   - **Quality Suggestions** (LOW): Missing tests, error handling, etc.
   - **Verdict**: APPROVED/NEEDS_REVISION/REJECTED based on HIGH severity count

Be thorough and constructive. Reference specific files and provide actionable recommendations.
EOF
)

# Run non-interactively with full auto mode
codex exec --full-auto "$PROMPT"
"#;

    let codex_verify = r#"#!/bin/bash
# Codex verify script
# Usage: ./codex-verify.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🧪 Verifying implementation with Codex: $CHANGE_ID"

# Use change-specific AGENTS.md context (generated dynamically by specter CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/specter/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Specter Verify Task

Verify the implementation for specter/changes/${CHANGE_ID}/.

## Instructions
1. Read the specs in specter/changes/${CHANGE_ID}/specs/
2. Check if the implementation meets all requirements
3. Run existing tests or create new tests as needed
4. Create specter/changes/${CHANGE_ID}/VERIFICATION.md with:
   - Test results
   - Spec compliance status
   - VERIFIED or FAILED verdict

Be thorough and test all requirements.
EOF
)

# Run non-interactively with full auto mode
codex exec --full-auto "$PROMPT"
"#;

    // Create codex-rechallenge.sh for session resumption
    let codex_rechallenge = r#"#!/bin/bash
# Codex re-challenge script (resumes previous session for cached context)
# Usage: ./codex-rechallenge.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Re-analyzing proposal with Codex (resuming session): $CHANGE_ID"

# Use change-specific AGENTS.md context (generated dynamically by specter CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/specter/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Specter Re-Challenge Task

A skeleton CHALLENGE.md has been updated at specter/changes/${CHANGE_ID}/CHALLENGE.md.
The proposal has been revised based on previous feedback.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read the UPDATED proposal files in specter/changes/${CHANGE_ID}/:
   - proposal.md (revised)
   - tasks.md (revised)
   - diagrams.md (revised)
   - specs/*.md (revised)

3. Re-fill the CHALLENGE.md with your findings:
   - **Internal Consistency Issues** (HIGH): Check if revised proposal docs now match each other
   - **Code Alignment Issues** (MEDIUM/LOW): Check alignment with existing code
     - If proposal mentions "refactor" or "BREAKING", note deviations as intentional
   - **Quality Suggestions** (LOW): Missing tests, error handling, etc.
   - **Verdict**: APPROVED/NEEDS_REVISION/REJECTED based on HIGH severity count

Focus on whether the previous issues have been addressed.
EOF
)

# Resume the previous session to reuse cached codebase context
codex resume --last "$PROMPT"
"#;

    std::fs::write(scripts_dir.join("gemini-proposal.sh"), gemini_proposal)?;
    std::fs::write(scripts_dir.join("gemini-reproposal.sh"), gemini_reproposal)?;
    std::fs::write(scripts_dir.join("codex-challenge.sh"), codex_challenge)?;
    std::fs::write(scripts_dir.join("codex-rechallenge.sh"), codex_rechallenge)?;
    std::fs::write(scripts_dir.join("codex-verify.sh"), codex_verify)?;

    // Placeholder for claude-implement.sh (not updated yet)
    let claude_implement = r#"#!/bin/bash
# Claude implement script
# Usage: ./claude-implement.sh <change-id>

CHANGE_ID="$1"

echo "🎨 Implementing: $CHANGE_ID"
echo "⚠️  This script is a placeholder - implementation happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-implement.sh"), claude_implement)?;

    // Placeholder for claude-fix.sh (not updated yet)
    let claude_fix = r#"#!/bin/bash
# Claude fix script
# Usage: ./claude-fix.sh <change-id>

CHANGE_ID="$1"

echo "🔧 Fixing issues: $CHANGE_ID"
echo "⚠️  This script is a placeholder - fixing happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-fix.sh"), claude_fix)?;

    // Make scripts executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in &[
            "gemini-proposal.sh",
            "gemini-reproposal.sh",
            "codex-challenge.sh",
            "codex-rechallenge.sh",
            "codex-verify.sh",
            "claude-implement.sh",
            "claude-fix.sh",
        ] {
            let path = scripts_dir.join(script);
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }
    }

    Ok(())
}

fn install_gemini_commands(gemini_dir: &Path) -> Result<()> {
    let commands_dir = gemini_dir.join("commands/specter");

    // Install command definitions
    std::fs::write(commands_dir.join("proposal.toml"), GEMINI_PROPOSAL)?;
    std::fs::write(commands_dir.join("reproposal.toml"), GEMINI_REPROPOSAL)?;

    println!("   ✓ gemini specter:proposal");
    println!("   ✓ gemini specter:reproposal");

    // Install settings
    std::fs::write(gemini_dir.join("settings.json"), GEMINI_SETTINGS)?;
    println!("   ✓ settings.json");

    Ok(())
}

fn install_codex_prompts(prompts_dir: &Path) -> Result<()> {
    // Create directory if it doesn't exist
    std::fs::create_dir_all(prompts_dir)?;

    let challenge_path = prompts_dir.join("specter-challenge.md");
    let verify_path = prompts_dir.join("specter-verify.md");

    // Check if prompts already exist
    let challenge_exists = challenge_path.exists();
    let verify_exists = verify_path.exists();

    if challenge_exists || verify_exists {
        println!();
        println!(
            "   {} Codex prompts already exist in ~/.codex/prompts/",
            "⚠️".yellow()
        );

        // Try to use interactive prompt, fall back to default if not available
        use dialoguer::Confirm;
        let overwrite = match Confirm::new()
            .with_prompt("Overwrite existing prompts?")
            .default(false)
            .interact()
        {
            Ok(response) => response,
            Err(_) => {
                // Not a terminal or dialoguer failed - use default (don't overwrite)
                println!("   (non-interactive mode: keeping existing prompts)");
                false
            }
        };

        if !overwrite {
            println!("   Skipping Codex prompts installation");
            return Ok(());
        }
    }

    // Install prompt files
    std::fs::write(&challenge_path, CODEX_CHALLENGE)?;
    std::fs::write(&verify_path, CODEX_VERIFY)?;

    println!("   ✓ codex specter-challenge (installed to ~/.codex/prompts/)");
    println!("   ✓ codex specter-verify (installed to ~/.codex/prompts/)");

    Ok(())
}
