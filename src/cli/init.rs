use crate::{models::AgentdConfig, Result};
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

// Current version for tracking upgrades
const AGENTD_VERSION: &str = env!("CARGO_PKG_VERSION");

// Claude Code Skills
const SKILL_PROPOSAL: &str = include_str!("../../templates/skills/agentd-proposal/SKILL.md");
const SKILL_CHALLENGE: &str = include_str!("../../templates/skills/agentd-challenge/SKILL.md");
const SKILL_REPROPOSAL: &str = include_str!("../../templates/skills/agentd-reproposal/SKILL.md");
const SKILL_IMPLEMENT: &str = include_str!("../../templates/skills/agentd-implement/SKILL.md");
const SKILL_REVIEW: &str = include_str!("../../templates/skills/agentd-review/SKILL.md");
const SKILL_FIX: &str = include_str!("../../templates/skills/agentd-fix/SKILL.md");
const SKILL_ARCHIVE: &str = include_str!("../../templates/skills/agentd-archive/SKILL.md");

// Gemini Commands
const GEMINI_PROPOSAL: &str = include_str!("../../templates/gemini/commands/agentd/proposal.toml");
const GEMINI_REPROPOSAL: &str =
    include_str!("../../templates/gemini/commands/agentd/reproposal.toml");
const GEMINI_FILLBACK: &str = include_str!("../../templates/gemini/commands/agentd/fillback.toml");
const GEMINI_SETTINGS: &str = include_str!("../../templates/gemini/settings.json");

// Codex Prompts
const CODEX_CHALLENGE: &str = include_str!("../../templates/codex/prompts/agentd-challenge.md");
const CODEX_REVIEW: &str = include_str!("../../templates/codex/prompts/agentd-review.md");

// Project Context Template
const PROJECT_TEMPLATE: &str = include_str!("../../templates/project.md");

// CLAUDE.md Template for target projects
const CLAUDE_TEMPLATE: &str = include_str!("../../templates/CLAUDE.md");

// Prompt for generating project.md
const PROJECT_INIT_PROMPT: &str = r#"Analyze this codebase and generate a project.md file.

Output ONLY the markdown content, no explanations or code blocks.

Format:
# Project Context

## Overview
<!-- One sentence: What does this project do? -->

## Tech Stack
- Language: <detected language>
- Framework: <detected framework or "None">
- Key libraries: <main dependencies>

## Conventions
- Error handling: <detected patterns>
- Naming: <detected style e.g. camelCase, snake_case>
- Testing: <detected test framework>

## Key Patterns
<!-- Important abstractions and patterns to follow -->

## Architecture
<!-- Brief module/component overview - keep concise -->

Analyze package.json, Cargo.toml, go.mod, pyproject.toml, or similar config files.
Look at src/ structure and key files to understand architecture.
Keep each section concise (2-4 lines max).
"#;

// AI Context Files (GEMINI.md and AGENTS.md) are now generated dynamically per change
// from templates/GEMINI.md and templates/AGENTS.md

pub async fn run(name: Option<&str>, force: bool) -> Result<()> {
    let project_root = env::current_dir()?;
    let agentd_dir = project_root.join("agentd");
    let claude_dir = project_root.join(".claude");
    let version_file = agentd_dir.join(".version");

    // Check if already initialized
    let is_initialized = agentd_dir.exists();

    if is_initialized && !force {
        // Check installed version
        let installed_version = std::fs::read_to_string(&version_file)
            .unwrap_or_else(|_| "unknown".to_string());

        println!("{}", "⚠️  Agentd is already initialized".yellow());
        println!("   Installed version: {}", installed_version.trim());
        println!("   Current version:   {}", AGENTD_VERSION);
        println!();
        println!("   Run {} to upgrade system files", "--force".cyan());
        println!("   (preserves specs, changes, archive, and config)");
        return Ok(());
    }

    if force && is_initialized {
        // Smart upgrade mode
        println!(
            "{}",
            format!("🔄 Upgrading Agentd to v{}...", AGENTD_VERSION).cyan().bold()
        );
        println!();
        run_upgrade(&project_root, &agentd_dir, &claude_dir, &version_file)?;
    } else {
        // Fresh install
        println!(
            "{}",
            format!("🎭 Initializing Agentd v{}...", AGENTD_VERSION).cyan().bold()
        );
        println!();
        run_fresh_install(name, &project_root, &agentd_dir, &claude_dir, &version_file)?;
    }

    Ok(())
}

/// Fresh install: create all directories and files
fn run_fresh_install(
    name: Option<&str>,
    project_root: &Path,
    agentd_dir: &Path,
    claude_dir: &Path,
    version_file: &Path,
) -> Result<()> {
    // Create directory structure
    println!("{}", "📁 Creating directory structure...".cyan());
    std::fs::create_dir_all(agentd_dir)?;
    std::fs::create_dir_all(agentd_dir.join("specs"))?;
    std::fs::create_dir_all(agentd_dir.join("changes"))?;
    std::fs::create_dir_all(agentd_dir.join("archive"))?;
    std::fs::create_dir_all(agentd_dir.join("scripts"))?;

    // Create project.md for project context
    let project_md_path = agentd_dir.join("project.md");
    std::fs::write(&project_md_path, PROJECT_TEMPLATE)?;
    println!("   ✓ agentd/project.md");

    // Try to auto-generate project.md with Gemini
    generate_project_md(&project_md_path);

    // Create Claude Code skills directory
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Create config
    let mut config = AgentdConfig::default();
    if let Some(n) = name {
        config.project_name = n.to_string();
    } else if let Some(dir_name) = project_root.file_name() {
        config.project_name = dir_name.to_string_lossy().to_string();
    }
    config.scripts_dir = PathBuf::from("agentd/scripts"); // Use relative path for portability
    config.save(project_root)?;

    // Install system files
    install_system_files(project_root, agentd_dir, claude_dir)?;

    // Generate CLAUDE.md with project context
    generate_claude_md(project_root, agentd_dir)?;

    // Write version file
    std::fs::write(version_file, AGENTD_VERSION)?;

    // Print success message
    print_init_success();

    Ok(())
}

/// Smart upgrade: only update system files, preserve user data
fn run_upgrade(
    project_root: &Path,
    agentd_dir: &Path,
    claude_dir: &Path,
    version_file: &Path,
) -> Result<()> {
    let old_version = std::fs::read_to_string(version_file)
        .unwrap_or_else(|_| "unknown".to_string());

    println!("{}", "📦 Preserving user data:".cyan());
    println!("   ✓ agentd/specs/     (untouched)");
    println!("   ✓ agentd/changes/   (untouched)");
    println!("   ✓ agentd/archive/   (untouched)");
    println!("   ✓ agentd/config.toml (untouched)");
    println!("   ✓ agentd/project.md (untouched)");
    println!();

    // Ensure scripts directory exists
    std::fs::create_dir_all(agentd_dir.join("scripts"))?;

    // Install/update system files
    install_system_files(project_root, agentd_dir, claude_dir)?;

    // Regenerate CLAUDE.md with project context
    generate_claude_md(project_root, agentd_dir)?;

    // Write new version
    std::fs::write(version_file, AGENTD_VERSION)?;

    println!();
    println!(
        "{}",
        format!("✅ Upgraded from {} to {}", old_version.trim(), AGENTD_VERSION)
            .green()
            .bold()
    );

    Ok(())
}

/// Install/update all system files (scripts, skills, commands, prompts)
fn install_system_files(
    project_root: &Path,
    agentd_dir: &Path,
    claude_dir: &Path,
) -> Result<()> {
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Install Claude Code Skills
    println!("{}", "🤖 Updating Claude Code Skills...".cyan());
    install_claude_skills(&skills_dir)?;

    // Install Gemini Commands (project-specific)
    println!();
    println!("{}", "🤖 Updating Gemini Commands...".cyan());
    let gemini_dir = project_root.join(".gemini");
    std::fs::create_dir_all(gemini_dir.join("commands/agentd"))?;
    install_gemini_commands(&gemini_dir)?;

    // Install Codex Prompts (user-space)
    println!();
    println!("{}", "🤖 Updating Codex Prompts...".cyan());
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let codex_prompts_dir = PathBuf::from(home_dir).join(".codex/prompts");
    install_codex_prompts(&codex_prompts_dir)?;

    // Create/update helper scripts
    println!();
    println!("{}", "📜 Updating helper scripts...".cyan());
    create_helper_scripts(&agentd_dir.join("scripts"))?;

    // Install shell completions
    println!();
    println!("{}", "🐚 Installing shell completions...".cyan());
    install_shell_completions()?;

    Ok(())
}

/// Print success message for fresh install
fn print_init_success() {
    println!();
    println!("{}", "✅ Agentd initialized successfully!".green().bold());
    println!();
    println!("{}", "📁 Structure:".cyan());
    println!("   agentd/                   - Main Agentd directory");
    println!(
        "   {}       - Project context (fill with AI)",
        "agentd/project.md".yellow()
    );
    println!("   agentd/specs/             - Main specifications");
    println!("   agentd/changes/           - Active changes");
    println!("   agentd/archive/           - Completed changes");
    println!("   .claude/skills/           - 7 Skills installed");
    println!("   .gemini/commands/agentd/  - 2 Gemini commands");
    println!("   ~/.codex/prompts/         - 2 Codex prompts");
    println!();

    println!("{}", "🤖 AI Commands Installed:".cyan().bold());
    println!(
        "   {} - Proposal generation",
        "gemini agentd:proposal".green()
    );
    println!(
        "   {} - Proposal refinement",
        "gemini agentd:reproposal".green()
    );
    println!("   {} - Code review", "codex agentd-challenge".green());
    println!("   {} - Test generation", "codex agentd-review".green());
    println!();

    println!(
        "{}",
        "🎯 Available Skills (use in Claude Code):".cyan().bold()
    );
    println!(
        "   {} - Generate proposal with Gemini",
        "/agentd:proposal".green()
    );
    println!(
        "   {} - Challenge proposal with Codex",
        "/agentd:challenge".green()
    );
    println!(
        "   {} - Refine based on feedback",
        "/agentd:reproposal".green()
    );
    println!(
        "   {} - Implement with Claude",
        "/agentd:implement".green()
    );
    println!("   {} - Verify with tests", "/agentd:review".green());
    println!("   {} - Fix verification failures", "/agentd:fix".green());
    println!(
        "   {} - Archive completed change",
        "/agentd:archive".green()
    );
    println!();

    println!("{}", "⏭️  Next Steps:".yellow().bold());
    println!(
        "   1. {} Fill project context with AI:",
        "📝".cyan()
    );
    println!(
        "      {}",
        "\"Read agentd/project.md and help me fill it out\"".cyan()
    );
    println!();
    println!("   2. Start your first proposal:");
    println!(
        "      {}",
        "/agentd:proposal my-feature \"Add awesome feature\"".cyan()
    );
}

/// Try to generate project.md using Gemini, or print prompt if unavailable
fn generate_project_md(project_md_path: &Path) {
    println!();
    println!("{}", "🤖 Generating project context...".cyan());

    // Check if gemini CLI is available
    let gemini_available = Command::new("gemini")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if gemini_available {
        // Run gemini to generate project.md
        println!("   Running Gemini to analyze codebase...");

        let output = Command::new("gemini")
            .args(["prompt", "-m", "gemini-2.0-flash", PROJECT_INIT_PROMPT])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let raw_content = String::from_utf8_lossy(&result.stdout);

                // Extract markdown content - find # Project Context and take from there
                let content = if let Some(start) = raw_content.find("# Project Context") {
                    let section = &raw_content[start..];
                    // Remove trailing code blocks or extra text
                    if let Some(end) = section.find("\n```\n") {
                        &section[..end]
                    } else {
                        section.trim_end_matches("```").trim()
                    }
                } else {
                    // Try to strip code blocks
                    raw_content
                        .trim()
                        .trim_start_matches("```markdown")
                        .trim_start_matches("```md")
                        .trim_start_matches("```")
                        .trim_end_matches("```")
                        .trim()
                };

                // Only write if we got meaningful content with actual data
                let has_content = content.contains("## Overview")
                    && content.len() > 200
                    && (content.contains("Language:") || content.contains("- Language"));

                if has_content && !content.contains("<!-- One sentence") {
                    if let Err(e) = std::fs::write(project_md_path, content) {
                        println!("   {} Failed to write: {}", "⚠️".yellow(), e);
                    } else {
                        println!("   {} project.md auto-generated!", "✓".green());
                        return;
                    }
                } else {
                    println!("   {} Gemini output incomplete, using template", "⚠️".yellow());
                }
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("   {} Gemini failed: {}", "⚠️".yellow(), stderr.trim());
            }
            Err(e) => {
                println!("   {} Failed to run Gemini: {}", "⚠️".yellow(), e);
            }
        }
    } else {
        println!("   {} Gemini CLI not found", "⚠️".yellow());
    }

    // Print prompt for manual use
    println!();
    println!("{}", "📋 To generate project.md manually, run:".yellow());
    println!();
    println!("   gemini prompt -m gemini-2.0-flash \\");
    println!("     \"{}\" > agentd/project.md", "Analyze this codebase and generate project.md with Overview, Tech Stack, Conventions, Key Patterns, Architecture sections");
    println!();
    println!("   Or ask Claude Code:");
    println!("     {}", "\"Read agentd/project.md and help me fill it out\"".cyan());
}

// Agentd section markers
const AGENTD_START_MARKER: &str = "<!-- agentd:start -->";
const AGENTD_END_MARKER: &str = "<!-- agentd:end -->";

/// Extract the Agentd section from template (between markers)
fn get_agentd_section() -> &'static str {
    let start = CLAUDE_TEMPLATE.find(AGENTD_START_MARKER).unwrap_or(0);
    let end = CLAUDE_TEMPLATE
        .find(AGENTD_END_MARKER)
        .map(|i| i + AGENTD_END_MARKER.len())
        .unwrap_or(CLAUDE_TEMPLATE.len());
    &CLAUDE_TEMPLATE[start..end]
}

/// Remove old Agentd sections (without markers) from content
fn remove_old_agentd_sections(content: &str) -> String {
    let mut result = content.to_string();

    // Pattern 1: "## Agentd Workflow" section (old format)
    if let Some(start) = result.find("## Agentd Workflow") {
        // Find the next ## heading or end of file
        let after_start = &result[start + 18..]; // skip "## Agentd Workflow"
        if let Some(next_heading) = after_start.find("\n## ") {
            let end = start + 18 + next_heading + 1; // +1 to keep the newline before next heading
            result = format!("{}{}", &result[..start], &result[end..]);
        } else {
            // No next heading - remove to end
            result = result[..start].trim_end().to_string();
        }
    }

    // Pattern 2: "## File Structure" section with agentd paths (old format)
    if let Some(start) = result.find("## File Structure") {
        let section_content = &result[start..];
        // Only remove if it contains agentd-specific content
        if section_content.contains("agentd/project.md") || section_content.contains("agentd/specs/") {
            let after_start = &result[start + 17..]; // skip "## File Structure"
            if let Some(next_heading) = after_start.find("\n## ") {
                let end = start + 17 + next_heading + 1;
                result = format!("{}{}", &result[..start], &result[end..]);
            } else if let Some(next_heading) = after_start.find("\n# ") {
                let end = start + 17 + next_heading + 1;
                result = format!("{}{}", &result[..start], &result[end..]);
            } else {
                result = result[..start].trim_end().to_string();
            }
        }
    }

    // Clean up multiple consecutive newlines
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    result
}

/// Generate or update CLAUDE.md with Agentd section (upsert mode)
fn generate_claude_md(project_root: &Path, agentd_dir: &Path) -> Result<()> {
    let project_md_path = agentd_dir.join("project.md");
    let claude_md_path = project_root.join("CLAUDE.md");

    let agentd_section = get_agentd_section();

    if claude_md_path.exists() {
        // CLAUDE.md exists - upsert the Agentd section
        let existing_content = std::fs::read_to_string(&claude_md_path)?;

        // First, remove old format sections (without markers)
        let cleaned_content = remove_old_agentd_sections(&existing_content);

        let new_content = if let (Some(start), Some(end)) = (
            cleaned_content.find(AGENTD_START_MARKER),
            cleaned_content.find(AGENTD_END_MARKER),
        ) {
            // Markers exist - replace content between them
            let before = &cleaned_content[..start];
            let after = &cleaned_content[end + AGENTD_END_MARKER.len()..];
            format!("{}{}{}", before, agentd_section, after)
        } else {
            // No markers - prepend Agentd section after first heading or at top
            if let Some(first_newline) = cleaned_content.find('\n') {
                let first_line = &cleaned_content[..first_newline];
                if first_line.starts_with('#') {
                    // Insert after the first heading
                    let after_heading = &cleaned_content[first_newline..];
                    format!("{}\n\n{}{}", first_line, agentd_section, after_heading)
                } else {
                    // Prepend at top
                    format!("{}\n\n{}", agentd_section, cleaned_content)
                }
            } else {
                format!("{}\n\n{}", agentd_section, cleaned_content)
            }
        };

        // Check if content changed
        if new_content.trim() == existing_content.trim() {
            println!("   {} CLAUDE.md (up to date)", "✓".green());
        } else {
            std::fs::write(&claude_md_path, new_content)?;
            println!("   {} CLAUDE.md (updated)", "✓".green());
        }
    } else {
        // CLAUDE.md doesn't exist - create with full template
        let project_content = std::fs::read_to_string(&project_md_path)
            .unwrap_or_else(|_| PROJECT_TEMPLATE.to_string());

        let claude_content = CLAUDE_TEMPLATE.replace("{{PROJECT_CONTEXT}}", &project_content);
        std::fs::write(&claude_md_path, claude_content)?;
        println!("   {} CLAUDE.md (created)", "✓".green());
    }

    Ok(())
}

/// Check if upgrade is available and optionally auto-upgrade
/// Returns true if auto-upgrade was performed
pub fn check_and_auto_upgrade(auto_upgrade: bool) -> bool {
    let project_root = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let version_file = project_root.join("agentd/.version");

    // Not initialized, nothing to upgrade
    if !version_file.exists() {
        return false;
    }

    let installed_version = std::fs::read_to_string(&version_file)
        .unwrap_or_else(|_| "0.0.0".to_string());
    let installed_version = installed_version.trim();

    // Compare versions
    if installed_version == AGENTD_VERSION {
        return false; // Already up to date
    }

    // Version mismatch detected
    if auto_upgrade {
        println!(
            "{}",
            format!(
                "🔄 Auto-upgrading Agentd: {} → {}",
                installed_version, AGENTD_VERSION
            )
            .cyan()
        );

        let agentd_dir = project_root.join("agentd");
        let claude_dir = project_root.join(".claude");

        if let Err(e) = run_upgrade(&project_root, &agentd_dir, &claude_dir, &version_file) {
            eprintln!("{}", format!("⚠️  Auto-upgrade failed: {}", e).yellow());
            return false;
        }

        println!();
        return true;
    } else {
        // Just notify
        println!(
            "{}",
            format!(
                "💡 Agentd update available: {} → {} (run {} to upgrade)",
                installed_version,
                AGENTD_VERSION,
                "agentd init --force".cyan()
            )
            .yellow()
        );
        println!();
        return false;
    }
}

/// Get installed version (for display purposes)
pub fn get_installed_version() -> Option<String> {
    let project_root = env::current_dir().ok()?;
    let version_file = project_root.join("agentd/.version");
    std::fs::read_to_string(&version_file).ok().map(|v| v.trim().to_string())
}

/// Get current CLI version
pub fn get_current_version() -> &'static str {
    AGENTD_VERSION
}

fn install_claude_skills(skills_dir: &Path) -> Result<()> {
    let skills = vec![
        ("proposal", SKILL_PROPOSAL),
        ("challenge", SKILL_CHALLENGE),
        ("reproposal", SKILL_REPROPOSAL),
        ("implement", SKILL_IMPLEMENT),
        ("review", SKILL_REVIEW),
        ("fix", SKILL_FIX),
        ("archive", SKILL_ARCHIVE),
    ];

    for (name, content) in skills {
        let skill_dir = skills_dir.join(format!("agentd-{}", name));
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(skill_dir.join("SKILL.md"), content)?;
        println!("   ✓ agentd-{}", name);
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

# Use change-specific GEMINI.md context (generated dynamically by agentd CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## User Request
${DESCRIPTION}

## Instructions
Create proposal files in agentd/changes/${CHANGE_ID}/.
EOF
)

# Call Gemini CLI with pre-defined command
echo "$CONTEXT" | gemini agentd:proposal --output-format stream-json
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

# Use change-specific GEMINI.md context (generated dynamically by agentd CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## Instructions
Read agentd/changes/${CHANGE_ID}/CHALLENGE.md and fix all HIGH and MEDIUM severity issues.
EOF
)

# Call Gemini CLI with pre-defined command
# Use --resume latest to reuse the proposal session (cached codebase context)
echo "$CONTEXT" | gemini agentd:reproposal --resume latest --output-format stream-json
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

# Use change-specific AGENTS.md context (generated dynamically by agentd CLI)
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Agentd Challenge Task

A skeleton CHALLENGE.md has been created at agentd/changes/${CHANGE_ID}/CHALLENGE.md.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read all proposal files in agentd/changes/${CHANGE_ID}/:
   - proposal.md
   - tasks.md
   - specs/*.md (contains Mermaid diagrams, JSON Schema, interfaces, acceptance criteria)

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

# Run with JSON streaming and parse output
cd "$PROJECT_ROOT" && codex exec --full-auto --json "$PROMPT" | while IFS= read -r line; do
  type=$(echo "$line" | jq -r '.type // empty' 2>/dev/null)
  case "$type" in
    item.completed)
      item_type=$(echo "$line" | jq -r '.item.type // empty' 2>/dev/null)
      case "$item_type" in
        reasoning)
          text=$(echo "$line" | jq -r '.item.text // empty' 2>/dev/null)
          [ -n "$text" ] && echo "💭 $text"
          ;;
        command_execution)
          cmd=$(echo "$line" | jq -r '.item.command // empty' 2>/dev/null)
          status=$(echo "$line" | jq -r '.item.status // empty' 2>/dev/null)
          [ -n "$cmd" ] && echo "⚡ $cmd ($status)"
          ;;
        agent_message)
          # Final message - just note completion
          echo "✅ Challenge analysis complete"
          ;;
      esac
      ;;
    turn.completed)
      tokens=$(echo "$line" | jq -r '.usage.input_tokens // 0' 2>/dev/null)
      echo "📊 Tokens used: $tokens"
      ;;
  esac
done
"#;

    let codex_review = r#"#!/bin/bash
# Codex code review script with test execution and security scanning
# Usage: ./codex-review.sh <change-id> <iteration>
set -euo pipefail

CHANGE_ID="$1"
ITERATION="${2:-0}"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Reviewing code with Codex (Iteration $ITERATION): $CHANGE_ID"

# Step 1: Run tests
echo "🧪 Running tests..."
TEST_OUTPUT=$(cargo test 2>&1 || true)
TEST_STATUS=$?

# Step 2: Run security scans
echo "🔒 Running security scans..."

# Rust: cargo audit (if available)
AUDIT_OUTPUT=""
if command -v cargo-audit &> /dev/null; then
    AUDIT_OUTPUT=$(cargo audit 2>&1 || true)
fi

# Universal: semgrep (if available)
SEMGREP_OUTPUT=""
if command -v semgrep &> /dev/null; then
    SEMGREP_OUTPUT=$(semgrep --config=auto --json 2>&1 || true)
fi

# Clippy with security lints
CLIPPY_OUTPUT=""
CLIPPY_OUTPUT=$(cargo clippy -- -W clippy::all -W clippy::pedantic 2>&1 || true)

# Step 3: Save outputs to temp files for Codex to read
TEMP_DIR=$(mktemp -d)
echo "$TEST_OUTPUT" > "$TEMP_DIR/test_output.txt"
echo "$AUDIT_OUTPUT" > "$TEMP_DIR/audit_output.txt"
echo "$SEMGREP_OUTPUT" > "$TEMP_DIR/semgrep_output.txt"
echo "$CLIPPY_OUTPUT" > "$TEMP_DIR/clippy_output.txt"

# Use change-specific AGENTS.md context
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Agentd Code Review Task (Iteration $ITERATION)

Review the implementation for agentd/changes/${CHANGE_ID}/.

## Available Data
- Test results: $TEMP_DIR/test_output.txt
- Security audit: $TEMP_DIR/audit_output.txt
- Semgrep scan: $TEMP_DIR/semgrep_output.txt
- Clippy output: $TEMP_DIR/clippy_output.txt

## Instructions
1. Read proposal.md, tasks.md, specs/ to understand requirements
2. Read implemented code (search for new/modified files)
3. **Analyze test results** from test_output.txt:
   - Parse test pass/fail status
   - Identify failing tests and reasons
   - Calculate coverage if available
4. **Analyze security scan results**:
   - Parse cargo audit for dependency vulnerabilities
   - Parse semgrep for security patterns
   - Parse clippy for code quality and security warnings
5. Review code quality, best practices, and requirement compliance
6. Fill agentd/changes/${CHANGE_ID}/REVIEW.md with comprehensive findings

## Review Focus
1. **Test Results (HIGH)**: Are all tests passing? Coverage adequate?
2. **Security (HIGH)**: Any vulnerabilities from tools? Security best practices?
3. **Best Practices (HIGH)**: Performance, error handling, style
4. **Requirement Compliance (HIGH)**: Does code match proposal/specs?
5. **Consistency (MEDIUM)**: Does code follow existing patterns?
6. **Test Quality (MEDIUM)**: Are tests comprehensive and well-written?

## Severity Guidelines
- **HIGH**: Failing tests, security vulnerabilities, missing features, wrong behavior
- **MEDIUM**: Style inconsistencies, missing tests, minor improvements
- **LOW**: Suggestions, nice-to-haves

## Verdict Guidelines
- **APPROVED**: All tests pass, no HIGH issues (LOW/MEDIUM issues acceptable)
- **NEEDS_CHANGES**: Some tests fail or HIGH/MEDIUM issues exist (fixable)
- **MAJOR_ISSUES**: Many failing tests or critical security issues

Be thorough but fair. Include iteration number in REVIEW.md.
EOF
)

# Run with JSON streaming
cd "$PROJECT_ROOT" && codex exec --full-auto --json "$PROMPT" | while IFS= read -r line; do
  type=$(echo "$line" | jq -r '.type // empty' 2>/dev/null)
  case "$type" in
    item.completed)
      item_type=$(echo "$line" | jq -r '.item.type // empty' 2>/dev/null)
      case "$item_type" in
        reasoning)
          text=$(echo "$line" | jq -r '.item.text // empty' 2>/dev/null)
          [ -n "$text" ] && echo "💭 $text"
          ;;
        command_execution)
          cmd=$(echo "$line" | jq -r '.item.command // empty' 2>/dev/null)
          status=$(echo "$line" | jq -r '.item.status // empty' 2>/dev/null)
          [ -n "$cmd" ] && echo "⚡ $cmd ($status)"
          ;;
        agent_message)
          echo "✅ Review analysis complete"
          ;;
      esac
      ;;
    turn.completed)
      tokens=$(echo "$line" | jq -r '.usage.input_tokens // 0' 2>/dev/null)
      echo "📊 Tokens used: $tokens"
      ;;
  esac
done

# Cleanup temp files
rm -rf "$TEMP_DIR"

echo "✅ Review complete (Iteration $ITERATION)"
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

# Use change-specific AGENTS.md context (generated dynamically by agentd CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Agentd Re-Challenge Task

A skeleton CHALLENGE.md has been updated at agentd/changes/${CHANGE_ID}/CHALLENGE.md.
The proposal has been revised based on previous feedback.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read the UPDATED proposal files in agentd/changes/${CHANGE_ID}/:
   - proposal.md (revised)
   - tasks.md (revised)
   - specs/*.md (revised - with Mermaid diagrams, JSON Schema, interfaces, acceptance criteria)

3. Re-fill the CHALLENGE.md with your findings:
   - **Internal Consistency Issues** (HIGH): Check if revised proposal docs now match each other
   - **Code Alignment Issues** (MEDIUM/LOW): Check alignment with existing code
     - If proposal mentions "refactor" or "BREAKING", note deviations as intentional
   - **Quality Suggestions** (LOW): Missing tests, error handling, etc.
   - **Verdict**: APPROVED/NEEDS_REVISION/REJECTED based on HIGH severity count

Focus on whether the previous issues have been addressed.
EOF
)

# Run with JSON streaming (codex resume doesn't support --json, use exec instead)
cd "$PROJECT_ROOT" && codex exec --full-auto --json "$PROMPT" | while IFS= read -r line; do
  type=$(echo "$line" | jq -r '.type // empty' 2>/dev/null)
  case "$type" in
    item.completed)
      item_type=$(echo "$line" | jq -r '.item.type // empty' 2>/dev/null)
      case "$item_type" in
        reasoning)
          text=$(echo "$line" | jq -r '.item.text // empty' 2>/dev/null)
          [ -n "$text" ] && echo "💭 $text"
          ;;
        command_execution)
          cmd=$(echo "$line" | jq -r '.item.command // empty' 2>/dev/null)
          status=$(echo "$line" | jq -r '.item.status // empty' 2>/dev/null)
          [ -n "$cmd" ] && echo "⚡ $cmd ($status)"
          ;;
        agent_message)
          echo "✅ Re-challenge analysis complete"
          ;;
      esac
      ;;
    turn.completed)
      tokens=$(echo "$line" | jq -r '.usage.input_tokens // 0' 2>/dev/null)
      echo "📊 Tokens used: $tokens"
      ;;
  esac
done
"#;

    // gemini-fillback.sh for reverse-engineering specs
    let gemini_fillback = r#"#!/bin/bash
# Gemini fillback (reverse-engineer specs) script
# Usage: ./gemini-fillback.sh <change-id> <json-request>
set -euo pipefail

CHANGE_ID="$1"
JSON_REQUEST="$2"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🤖 Reverse-engineering specs with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by agentd CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/GEMINI.md"

# Parse JSON request
FILE_COUNT=$(echo "$JSON_REQUEST" | jq -r '.files | length')
PROMPT=$(echo "$JSON_REQUEST" | jq -r '.prompt')

echo "📊 Analyzing $FILE_COUNT source files..."

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## Task
${PROMPT}

## Source Files
The following source files have been scanned from the codebase:

EOF
)

# Add file information
for i in $(seq 0 $((FILE_COUNT - 1))); do
    FILE_PATH=$(echo "$JSON_REQUEST" | jq -r ".files[$i].path")
    CONTEXT="${CONTEXT}
- ${FILE_PATH}"
done

CONTEXT="${CONTEXT}

## Source Code
\`\`\`
"

# Add file contents
for i in $(seq 0 $((FILE_COUNT - 1))); do
    FILE_PATH=$(echo "$JSON_REQUEST" | jq -r ".files[$i].path")
    FILE_CONTENT=$(echo "$JSON_REQUEST" | jq -r ".files[$i].content")

    CONTEXT="${CONTEXT}
=== ${FILE_PATH} ===
${FILE_CONTENT}

"
done

CONTEXT="${CONTEXT}\`\`\`

## Instructions
Analyze the provided source code and generate:
1. proposal.md in agentd/changes/${CHANGE_ID}/proposal.md
2. Technical specifications in agentd/changes/${CHANGE_ID}/specs/*.md
3. tasks.md in agentd/changes/${CHANGE_ID}/tasks.md

The specifications should include:
- High-level architecture and design patterns used
- Key requirements and components
- Data models and interfaces
- Acceptance criteria based on code behavior

Focus on creating actionable, well-structured Agentd specifications that capture the technical design.
"

# Call Gemini CLI with pre-defined command
echo "$CONTEXT" | gemini agentd:fillback --output-format stream-json
"#;

    std::fs::write(scripts_dir.join("gemini-proposal.sh"), gemini_proposal)?;
    std::fs::write(scripts_dir.join("gemini-reproposal.sh"), gemini_reproposal)?;
    std::fs::write(scripts_dir.join("gemini-fillback.sh"), gemini_fillback)?;
    std::fs::write(scripts_dir.join("codex-challenge.sh"), codex_challenge)?;
    std::fs::write(scripts_dir.join("codex-rechallenge.sh"), codex_rechallenge)?;
    std::fs::write(scripts_dir.join("codex-review.sh"), codex_review)?;

    // Updated claude-implement.sh with emphasis on test writing
    let claude_implement = r#"#!/bin/bash
# Claude implement script - writes code AND tests
# Usage: ./claude-implement.sh <change-id>

CHANGE_ID="$1"
TASKS="${2:-}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🎨 Implementing with Claude: $CHANGE_ID"

PROMPT=$(cat << EOF
# Agentd Implement Task

Implement the proposal for agentd/changes/${CHANGE_ID}/.

## Instructions
1. Read proposal.md, tasks.md, and specs/
2. Implement ALL tasks in tasks.md (or only ${TASKS} if specified)
3. **Write tests for all implemented features** (unit + integration)
   - Test all spec scenarios (WHEN/THEN cases)
   - Include edge cases and error handling
   - Use existing test framework patterns
4. Update IMPLEMENTATION.md with progress notes

## Code Quality
- Follow existing code style and patterns
- Add proper error handling
- Include documentation comments where needed

**IMPORTANT**: Write comprehensive tests. Tests are as important as the code itself.
EOF
)

# This is a placeholder - actual implementation happens via Claude Code Skills
# When called from CLI, Claude IDE will handle the implementation
echo "⚠️  This script is a placeholder - implementation happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-implement.sh"), claude_implement)?;

    // claude-resolve.sh for fixing review issues
    let claude_resolve = r#"#!/bin/bash
# Claude resolve script - fixes issues from code review
# Usage: ./claude-resolve.sh <change-id>

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔧 Resolving review issues with Claude: $CHANGE_ID"

PROMPT=$(cat << EOF
# Agentd Resolve Reviews Task

Fix issues identified in code review for agentd/changes/${CHANGE_ID}/.

## Instructions
1. Read REVIEW.md to understand all issues
2. Fix ALL HIGH and MEDIUM severity issues:
   - Failing tests
   - Security vulnerabilities
   - Missing features
   - Wrong behavior
   - Style inconsistencies
   - Missing tests
3. Update code, tests, and documentation as needed
4. Update IMPLEMENTATION.md with resolution notes

Focus on HIGH severity issues first, then MEDIUM.
EOF
)

# This is a placeholder - actual resolution happens via Claude Code Skills
echo "⚠️  This script is a placeholder - resolution happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-resolve.sh"), claude_resolve)?;

    // Placeholder for claude-fix.sh (kept for backward compatibility)
    let claude_fix = r#"#!/bin/bash
# Claude fix script
# Usage: ./claude-fix.sh <change-id>

CHANGE_ID="$1"

echo "🔧 Fixing issues: $CHANGE_ID"
echo "⚠️  This script is a placeholder - fixing happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-fix.sh"), claude_fix)?;

    // gemini-merge-specs.sh for merging spec deltas
    let gemini_merge_specs = r#"#!/bin/bash
# Gemini merge specs script - merges delta specs back to main specs
# Usage: ./gemini-merge-specs.sh <change-id> <strategy> <spec-file>

set -euo pipefail

CHANGE_ID="$1"
STRATEGY="$2"
SPEC_FILE="$3"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔄 Merging specs with Gemini: $SPEC_FILE ($STRATEGY)"

# Use GEMINI.md context (generated by CLI)
export GEMINI_INSTRUCTIONS_FILE="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/GEMINI.md"

DELTA_SPEC="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/specs/$SPEC_FILE"
MAIN_SPEC="$PROJECT_ROOT/agentd/specs/$SPEC_FILE"

PROMPT=$(cat << EOF
# Agentd Archive: Spec Merging Task

Merge spec deltas back to main specification using **${STRATEGY}** strategy.

## Context
- Change ID: ${CHANGE_ID}
- Spec file: ${SPEC_FILE}
- Strategy: ${STRATEGY}
- Delta spec: agentd/changes/${CHANGE_ID}/specs/${SPEC_FILE}
- Main spec: agentd/specs/${SPEC_FILE}

## Your Task

Read both the delta spec and main spec (if it exists), then merge them according to the strategy:

**Full Rewrite**: Rewrite the entire spec incorporating all changes. Maintain consistent style and structure.

**Differential Merge**: Apply ONLY the specified changes. Preserve all other content exactly as-is.

**Hybrid**: Rewrite affected sections, preserve unaffected sections. Ensure smooth transitions.

## Output

Write the merged spec to: agentd/specs/${SPEC_FILE}

Ensure the output follows the spec schema with:
- Required headings: # Specification:, ## Overview, ## Requirements
- Requirement format: ### R\d+:
- Scenario format: #### Scenario:
- WHEN/THEN clauses in scenarios
EOF
)

gemini agentd:merge-specs --resume latest "$PROMPT"
"#;
    std::fs::write(scripts_dir.join("gemini-merge-specs.sh"), gemini_merge_specs)?;

    // gemini-changelog.sh for generating CHANGELOG entries
    let gemini_changelog = r#"#!/bin/bash
# Gemini CHANGELOG script - generates CHANGELOG entry
# Usage: ./gemini-changelog.sh <change-id>

set -euo pipefail

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "📝 Generating CHANGELOG entry: $CHANGE_ID"

PROMPT=$(cat << EOF
# Agentd Archive: CHANGELOG Generation

Generate a concise CHANGELOG entry for change: ${CHANGE_ID}

## Input Files

Read these files to understand what changed:
- agentd/changes/${CHANGE_ID}/proposal.md
- agentd/changes/${CHANGE_ID}/tasks.md
- agentd/changes/${CHANGE_ID}/specs/ (all spec files)

## Output Format

Use the Keep a Changelog format:

\`\`\`
## $(date +%Y-%m-%d): <Title> (${CHANGE_ID})
<1-2 sentence summary of what changed and why>
- Related specs: spec1.md, spec2.md
\`\`\`

## Requirements

- **1-2 sentences only** - Be concise
- Focus on **what** and **why** (not how)
- Use past tense (e.g., "Added", "Updated", "Fixed")
- List all affected spec files

## Example

\`\`\`
## 2026-01-13: Add OAuth 2.0 Authentication (add-oauth)
Added OAuth 2.0 support with Google and GitHub providers to enable social login. Includes automatic token refresh with 7-day expiry.
- Related specs: specs/auth/oauth.md, specs/auth/session.md
\`\`\`

## Output

Prepend the entry to: agentd/specs/CHANGELOG.md

If CHANGELOG.md doesn't exist, create it with:
\`\`\`
# CHANGELOG

All notable changes to this project's specifications will be documented in this file.

[Your generated entry here]
\`\`\`
EOF
)

gemini agentd:changelog "$PROMPT"
"#;
    std::fs::write(scripts_dir.join("gemini-changelog.sh"), gemini_changelog)?;

    // codex-archive-review.sh for quality review before archiving
    let codex_archive_review = r#"#!/bin/bash
# Codex archive review script - reviews merged specs and CHANGELOG for quality
# Usage: ./codex-archive-review.sh <change-id> <strategy>

set -euo pipefail

CHANGE_ID="$1"
STRATEGY="$2"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Reviewing archive quality with Codex: $CHANGE_ID"

# Create ARCHIVE_REVIEW.md skeleton
REVIEW_PATH="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/ARCHIVE_REVIEW.md"
cat > "$REVIEW_PATH" << 'SKELETON'
# Archive Quality Review

## Status: [ ] APPROVED | [ ] NEEDS_FIX | [ ] REJECTED

## Merged Specs Review

[Codex will fill this]

## CHANGELOG Review

[Codex will fill this]

## Overall Assessment

[Summary]

## Issues Found

[List if any]

## Recommendation

- [ ] APPROVED - Safe to archive
- [ ] NEEDS_FIX - Fix issues above
- [ ] REJECTED - Major issues
SKELETON

PROMPT=$(cat << EOF
# Agentd Archive: Quality Review Task

Review the merged specs and CHANGELOG before archiving change: ${CHANGE_ID}

## Context

You need to verify that Gemini correctly merged the spec deltas:
- **Delta specs**: agentd/changes/${CHANGE_ID}/specs/ (original changes)
- **Merged specs**: agentd/specs/ (after Gemini merge)
- **Strategy used**: ${STRATEGY}
- **CHANGELOG**: agentd/specs/CHANGELOG.md (latest entry should be for ${CHANGE_ID})

## Your Task

### 1. Compare Delta vs Merged Specs

**IMPORTANT**: Skip template files (files starting with underscore like _skeleton.md).

For each spec file in the delta (excluding templates):
1. Read the delta spec: agentd/changes/${CHANGE_ID}/specs/[file]
2. Read the merged spec: agentd/specs/[file]
3. Verify ALL changes from delta are present in merged spec
4. Check for hallucinations (content added by Gemini not in delta)
5. Check for omissions (content missing that should be present)

### 2. Format Validation

For each merged spec, verify:
- Proper headings: # Specification:, ## Overview, ## Requirements
- Requirement format: ### R\d+: [title]
- Scenario format: #### Scenario: [name]
- WHEN/THEN clauses present in all scenarios

### 3. CHANGELOG Verification

Check the latest CHANGELOG entry:
- Is it for ${CHANGE_ID}?
- Does it list all affected spec files?
- Is it concise (1-2 sentences)?
- Does it use past tense?
- Is the date correct?

### 4. Cross-File Validation

- All spec files from delta have corresponding merged files
- No duplicate requirement IDs across specs
- All cross-references are valid

### 5. Generate Report

Update the file: agentd/changes/${CHANGE_ID}/ARCHIVE_REVIEW.md

Fill in all sections with your findings:

1. **Status**: Mark ONE checkbox with [x]:
   - [x] APPROVED if all checks pass, no issues found
   - [x] NEEDS_FIX if minor issues (missing WHEN, formatting errors, incomplete CHANGELOG)
   - [x] REJECTED if major issues (missing requirements, corrupted specs, wrong content)

2. **Merged Specs Review**: For each spec file, list checks:
   - [x] Format valid / [ ] Issue: [description]
   - [x] Delta complete / [ ] Issue: [description]
   - [x] No hallucinations / [ ] Issue: [description]

3. **CHANGELOG Review**: List checks for CHANGELOG

4. **Overall Assessment**: Brief summary of findings

5. **Issues Found**: Number each issue with severity (HIGH/MEDIUM/LOW), category, and description:
   1. **HIGH**: specs/file.md - Missing requirement R3 from delta
   2. **MEDIUM**: CHANGELOG doesn't mention session timeout change

6. **Recommendation**: Explain which action to take

**Decision Criteria**:
- **APPROVED**: All checks pass, no issues found
- **NEEDS_FIX**: Minor issues that need fixing before archive
- **REJECTED**: Major issues that require manual intervention

Be strict. Quality is critical. Look for:
- Hallucinations: Content added by Gemini that wasn't in delta
- Omissions: Content from delta missing in merged spec
- Format violations: Incorrect heading levels, malformed requirements
- Logic errors: Contradictions, broken references

Now perform the review and update the ARCHIVE_REVIEW.md file.
EOF
)

cd "$PROJECT_ROOT" && codex exec --full-auto --json "$PROMPT" | while IFS= read -r line; do
  type=$(echo "$line" | jq -r '.type // empty' 2>/dev/null)
  case "$type" in
    item.completed)
      item_type=$(echo "$line" | jq -r '.item.type // empty' 2>/dev/null)
      case "$item_type" in
        reasoning)
          text=$(echo "$line" | jq -r '.item.text // empty' 2>/dev/null)
          [ -n "$text" ] && echo "💭 $text"
          ;;
        command_execution)
          cmd=$(echo "$line" | jq -r '.item.command // empty' 2>/dev/null)
          status=$(echo "$line" | jq -r '.item.status // empty' 2>/dev/null)
          [ -n "$cmd" ] && echo "⚡ $cmd ($status)"
          ;;
        agent_message)
          echo "✅ Archive review complete"
          ;;
      esac
      ;;
    turn.completed)
      tokens=$(echo "$line" | jq -r '.usage.input_tokens // 0' 2>/dev/null)
      echo "📊 Tokens used: $tokens"
      ;;
  esac
done

echo "✅ Review complete: $REVIEW_PATH"
"#;
    std::fs::write(scripts_dir.join("codex-archive-review.sh"), codex_archive_review)?;

    // Make scripts executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in &[
            "gemini-proposal.sh",
            "gemini-reproposal.sh",
            "gemini-fillback.sh",
            "gemini-merge-specs.sh",
            "gemini-changelog.sh",
            "codex-challenge.sh",
            "codex-rechallenge.sh",
            "codex-review.sh",
            "codex-archive-review.sh",
            "claude-implement.sh",
            "claude-resolve.sh",
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
    let commands_dir = gemini_dir.join("commands/agentd");

    // Install command definitions
    std::fs::write(commands_dir.join("proposal.toml"), GEMINI_PROPOSAL)?;
    std::fs::write(commands_dir.join("reproposal.toml"), GEMINI_REPROPOSAL)?;
    std::fs::write(commands_dir.join("fillback.toml"), GEMINI_FILLBACK)?;

    println!("   ✓ gemini agentd:proposal");
    println!("   ✓ gemini agentd:reproposal");
    println!("   ✓ gemini agentd:fillback");

    // Install settings
    std::fs::write(gemini_dir.join("settings.json"), GEMINI_SETTINGS)?;
    println!("   ✓ settings.json");

    Ok(())
}

/// Compute a simple checksum for content comparison
fn compute_checksum(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Install shell completions for supported shells
fn install_shell_completions() -> Result<()> {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    // Get current executable path
    let current_exe = std::env::current_exe()?;

    // Install zsh completions
    let zsh_completions_dir = PathBuf::from(&home_dir).join(".zsh/completions");
    std::fs::create_dir_all(&zsh_completions_dir)?;

    let zsh_completion_file = zsh_completions_dir.join("_agentd");

    // Generate zsh completions by running ourselves
    let output = Command::new(&current_exe)
        .args(["completions", "zsh"])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let completions = String::from_utf8_lossy(&result.stdout);
            std::fs::write(&zsh_completion_file, completions.as_bytes())?;
            println!("   ✓ zsh completions → ~/.zsh/completions/_agentd");

            // Check if fpath is configured
            let zshrc_path = PathBuf::from(&home_dir).join(".zshrc");
            let fpath_configured = if zshrc_path.exists() {
                std::fs::read_to_string(&zshrc_path)
                    .map(|content| content.contains(".zsh/completions"))
                    .unwrap_or(false)
            } else {
                false
            };

            if !fpath_configured {
                println!();
                println!("   {} Add to ~/.zshrc:", "💡".yellow());
                println!("      fpath=(~/.zsh/completions $fpath)");
                println!("      autoload -Uz compinit && compinit");
            }
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            println!("   {} zsh completions failed: {}", "⚠️".yellow(), stderr.trim());
        }
        Err(e) => {
            println!("   {} Failed to generate zsh completions: {}", "⚠️".yellow(), e);
        }
    }

    Ok(())
}

fn install_codex_prompts(prompts_dir: &Path) -> Result<()> {
    // Create directory if it doesn't exist
    std::fs::create_dir_all(prompts_dir)?;

    let challenge_path = prompts_dir.join("agentd-challenge.md");
    let review_path = prompts_dir.join("agentd-review.md");

    // Check if prompts already exist and compare checksums
    let challenge_same = challenge_path.exists()
        && std::fs::read_to_string(&challenge_path)
            .map(|content| compute_checksum(&content) == compute_checksum(CODEX_CHALLENGE))
            .unwrap_or(false);

    let review_same = review_path.exists()
        && std::fs::read_to_string(&review_path)
            .map(|content| compute_checksum(&content) == compute_checksum(CODEX_REVIEW))
            .unwrap_or(false);

    // If both are the same, skip silently
    if challenge_same && review_same {
        println!("   ✓ codex agentd-challenge (up to date)");
        println!("   ✓ codex agentd-review (up to date)");
        return Ok(());
    }

    // Check if any prompts exist but differ
    let challenge_exists = challenge_path.exists();
    let review_exists = review_path.exists();

    if (challenge_exists && !challenge_same) || (review_exists && !review_same) {
        println!();
        println!(
            "   {} Codex prompts differ from current version",
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
    std::fs::write(&review_path, CODEX_REVIEW)?;

    println!("   ✓ codex agentd-challenge (installed to ~/.codex/prompts/)");
    println!("   ✓ codex agentd-review (installed to ~/.codex/prompts/)");

    Ok(())
}
