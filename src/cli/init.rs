use crate::{models::AgentdConfig, Result};
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

// Current version for tracking upgrades
const AGENTD_VERSION: &str = env!("CARGO_PKG_VERSION");

// Claude Code Skills (high-level workflows only)
const SKILL_PLAN_CHANGE: &str = include_str!("../../templates/mainthread/skills/agentd-plan-change/SKILL.md");
const SKILL_IMPL_CHANGE: &str = include_str!("../../templates/mainthread/skills/agentd-impl-change/SKILL.md");
const SKILL_MERGE_CHANGE: &str = include_str!("../../templates/mainthread/skills/agentd-merge-change/SKILL.md");

// Project Context Template
const PROJECT_TEMPLATE: &str = include_str!("../../templates/project.md");

// Knowledge Base Template
const KNOWLEDGE_INDEX_TEMPLATE: &str = include_str!("../../templates/knowledge/index.md");

// CLAUDE.md Template for target projects
const CLAUDE_TEMPLATE: &str = include_str!("../../templates/mainthread/CLAUDE.md");

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
        let old_version_trimmed = old_version.trim();

        // Check for version downgrade
        if old_version_trimmed != "unknown" && old_version_trimmed != AGENTD_VERSION {
            if !super::update::is_newer(AGENTD_VERSION, old_version_trimmed) {
                println!(
                    "{}",
                    format!(
                        "⚠️  Cannot downgrade from {} to {}",
                        old_version_trimmed, AGENTD_VERSION
                    )
                    .yellow()
                    .bold()
                );
                println!();
                println!("   {} This would downgrade your Agentd installation.", "⚠️".yellow());
                println!("   {} Current CLI version: {}", "ℹ️".cyan(), AGENTD_VERSION.yellow());
                println!("   {} Installed version:  {}", "ℹ️".cyan(), old_version_trimmed.green());
                println!();
                println!(
                    "{}",
                    "💡 To upgrade, install a newer version of the CLI first:".yellow()
                );
                println!("   curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/agentd/main/install.sh | bash");
                println!();
                return Ok(());
            }
        }

        println!(
            "{}",
            format!("🔄 Updating Agentd {} → {}...", old_version_trimmed, AGENTD_VERSION).cyan().bold()
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
    std::fs::create_dir_all(agentd_dir.join("knowledge"))?;

    // Create project.md for project context
    let project_md_path = agentd_dir.join("project.md");
    std::fs::write(&project_md_path, PROJECT_TEMPLATE)?;
    println!("   ✓ agentd/project.md");

    // Create knowledge/index.md
    let knowledge_index_path = agentd_dir.join("knowledge/index.md");
    std::fs::write(&knowledge_index_path, KNOWLEDGE_INDEX_TEMPLATE)?;
    println!("   ✓ agentd/knowledge/index.md");

    // Print instructions for generating project.md
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
    println!("   ✓ agentd/knowledge/ (untouched)");

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

    // Ensure knowledge directory exists
    std::fs::create_dir_all(agentd_dir.join("knowledge"))?;
    let knowledge_index_path = agentd_dir.join("knowledge/index.md");
    if !knowledge_index_path.exists() {
        std::fs::write(&knowledge_index_path, KNOWLEDGE_INDEX_TEMPLATE)?;
        println!("   {} agentd/knowledge/index.md (created)", "✓".green());
    }

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

/// Install/update all system files (skills)
fn install_system_files(
    _project_root: &Path,
    _agentd_dir: &Path,
    claude_dir: &Path,
) -> Result<()> {
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Install Claude Code Skills
    println!("{}", "🤖 Updating Claude Code Skills...".cyan());
    install_claude_skills(&skills_dir)?;

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
    println!("   agentd/knowledge/         - System documentation");
    println!("   .claude/skills/           - 3 Skills installed");
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

/// Print prompt for generating project.md manually
fn generate_project_md(_project_md_path: &Path) {
    println!();
    println!("{}", "📋 To generate project.md, ask Claude Code:".yellow());
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
        // Old workflow skill names (renamed)
        "agentd-plan",
        "agentd-impl",
        "agentd-archive",
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
        ("plan-change", SKILL_PLAN_CHANGE),
        ("impl-change", SKILL_IMPL_CHANGE),
        ("merge-change", SKILL_MERGE_CHANGE),
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
