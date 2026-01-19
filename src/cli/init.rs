use crate::{models::AgentdConfig, Result};
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

// Current version for tracking upgrades
const AGENTD_VERSION: &str = env!("CARGO_PKG_VERSION");

// Claude Code Skills (high-level workflows only)
const SKILL_PLAN: &str = include_str!("../../templates/skills/agentd-plan/SKILL.md");
const SKILL_IMPL: &str = include_str!("../../templates/skills/agentd-impl/SKILL.md");
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

// Helper Scripts - Single source of truth from agentd/scripts/
// Shell scripts are no longer generated during init.
// Orchestrators now call CLI tools directly instead of using shell scripts.
// These constants are kept for reference but are no longer used.
// const SCRIPT_GEMINI_PROPOSAL: &str = include_str!("../../agentd/scripts/gemini-proposal.sh");
// const SCRIPT_GEMINI_REPROPOSAL: &str = include_str!("../../agentd/scripts/gemini-reproposal.sh");
// const SCRIPT_GEMINI_FILLBACK: &str = include_str!("../../agentd/scripts/gemini-fillback.sh");
// const SCRIPT_GEMINI_MERGE_SPECS: &str = include_str!("../../agentd/scripts/gemini-merge-specs.sh");
// const SCRIPT_GEMINI_CHANGELOG: &str = include_str!("../../agentd/scripts/gemini-changelog.sh");
// const SCRIPT_GEMINI_ARCHIVE_FIX: &str = include_str!("../../agentd/scripts/gemini-archive-fix.sh");
// const SCRIPT_CODEX_CHALLENGE: &str = include_str!("../../agentd/scripts/codex-challenge.sh");
// const SCRIPT_CODEX_RECHALLENGE: &str = include_str!("../../agentd/scripts/codex-rechallenge.sh");
// const SCRIPT_CODEX_REVIEW: &str = include_str!("../../agentd/scripts/codex-review.sh");
// const SCRIPT_CODEX_ARCHIVE_REVIEW: &str = include_str!("../../agentd/scripts/codex-archive-review.sh");
// const SCRIPT_CLAUDE_IMPLEMENT: &str = include_str!("../../agentd/scripts/claude-implement.sh");
// const SCRIPT_CLAUDE_RESOLVE: &str = include_str!("../../agentd/scripts/claude-resolve.sh");

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

pub async fn run(name: Option<&str>, _force: bool) -> Result<()> {
    let project_root = env::current_dir()?;
    let agentd_dir = project_root.join("agentd");
    let claude_dir = project_root.join(".claude");
    let version_file = agentd_dir.join(".version");

    // Check if already initialized
    let is_initialized = agentd_dir.exists();

    if is_initialized {
        // Update mode: overwrite system files, preserve project.md
        let old_version = std::fs::read_to_string(&version_file)
            .unwrap_or_else(|_| "unknown".to_string());
        println!(
            "{}",
            format!("🔄 Updating Agentd {} → {}...", old_version.trim(), AGENTD_VERSION).cyan().bold()
        );
        println!();
        run_update(name, &project_root, &agentd_dir, &claude_dir, &version_file)?;
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

/// Update mode: overwrite config.toml, preserve project.md, update system files
fn run_update(
    name: Option<&str>,
    project_root: &Path,
    agentd_dir: &Path,
    claude_dir: &Path,
    version_file: &Path,
) -> Result<()> {
    println!("{}", "📦 User data:".cyan());
    println!("   ✓ agentd/specs/     (untouched)");
    println!("   ✓ agentd/changes/   (untouched)");
    println!("   ✓ agentd/archive/   (untouched)");

    // Overwrite config.toml (opinionated defaults)
    let mut config = AgentdConfig::default();
    if let Some(n) = name {
        config.project_name = n.to_string();
    } else if let Some(dir_name) = project_root.file_name() {
        config.project_name = dir_name.to_string_lossy().to_string();
    }
    config.scripts_dir = PathBuf::from("agentd/scripts");
    config.save(project_root)?;
    println!("   {} agentd/config.toml (updated)", "✓".green());

    // Preserve project.md (user content) - only create if missing
    let project_md_path = agentd_dir.join("project.md");
    if project_md_path.exists() {
        println!("   ✓ agentd/project.md (preserved)");
    } else {
        std::fs::write(&project_md_path, PROJECT_TEMPLATE)?;
        println!("   {} agentd/project.md (created)", "✓".green());
    }
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
    println!("{}", "✅ Update complete!".green().bold());

    Ok(())
}

/// Install/update all system files (skills, commands, prompts)
fn install_system_files(
    project_root: &Path,
    _agentd_dir: &Path,  // Kept for backward compatibility
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

    // Note: Shell scripts are no longer generated. Orchestrators now call CLI tools directly.
    // The agentd/scripts/ directory is kept for backward compatibility and custom user scripts.

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
    println!("   .claude/skills/           - 3 Skills installed");
    println!("   .gemini/commands/agentd/  - 3 Gemini commands");
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
        "🎯 Primary Workflows (use in Claude Code):".cyan().bold()
    );
    println!(
        "   {} - Plan and validate proposal",
        "/agentd:plan".green().bold()
    );
    println!(
        "   {} - Implement and iterate",
        "/agentd:impl".green().bold()
    );
    println!(
        "   {} - Archive completed change",
        "/agentd:archive".green().bold()
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
    println!("   2. Start your first change:");
    println!(
        "      {}",
        "/agentd:plan my-feature \"Add awesome feature\"".cyan()
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

    // Compare versions - only upgrade if CLI version is newer than installed
    if installed_version == AGENTD_VERSION {
        return false; // Already up to date
    }

    // Check if CLI version is actually newer (not older)
    if !super::update::is_newer(AGENTD_VERSION, installed_version) {
        // CLI is older than installed - don't downgrade
        return false;
    }

    // CLI version is newer - upgrade
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

        if let Err(e) = run_update(None, &project_root, &agentd_dir, &claude_dir, &version_file) {
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
    // Remove deprecated skills
    let deprecated_skills = vec![
        "agentd-proposal",
        "agentd-challenge",
        "agentd-reproposal",
        "agentd-implement",
        "agentd-review",
        "agentd-resolve-reviews",
        "agentd-fix",
        "agentd-verify",
    ];

    for deprecated in &deprecated_skills {
        let deprecated_dir = skills_dir.join(deprecated);
        if deprecated_dir.exists() {
            std::fs::remove_dir_all(&deprecated_dir)?;
            println!("   {} {} (removed)", "✗".red(), deprecated);
        }
    }

    // Install current skills
    let skills = vec![
        ("plan", SKILL_PLAN),
        ("impl", SKILL_IMPL),
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


// No longer used - shell scripts are no longer generated during init.
// Orchestrators now call CLI tools directly instead of using shell scripts.
// The function has been removed. The agentd/scripts/ directory is kept for
// backward compatibility and custom user scripts only.

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
