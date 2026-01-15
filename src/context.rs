use crate::Result;
use colored::Colorize;
use dialoguer::Select;
use std::path::Path;
use walkdir::WalkDir;

const GEMINI_TEMPLATE: &str = include_str!("../templates/GEMINI.md");
const AGENTS_TEMPLATE: &str = include_str!("../templates/AGENTS.md");

// Skeleton templates (embedded defaults, can be overridden)
const PROPOSAL_SKELETON: &str = include_str!("../templates/skeletons/proposal.md");
const TASKS_SKELETON: &str = include_str!("../templates/skeletons/tasks.md");
const SPEC_SKELETON: &str = include_str!("../templates/skeletons/spec.md");
const CHALLENGE_SKELETON: &str = include_str!("../templates/skeletons/challenge.md");
const REVIEW_SKELETON: &str = include_str!("../templates/skeletons/review.md");
const ARCHIVE_REVIEW_SKELETON: &str = include_str!("../templates/skeletons/archive_review.md");

/// Context phase determines which project.md sections to inject
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPhase {
    /// Design phase: Overview, Architecture, Key Patterns
    Proposal,
    /// Design review: Overview, Architecture, Key Patterns
    Challenge,
    /// Code writing: Overview, Tech Stack, Conventions, Key Patterns
    Implement,
    /// Code review: Overview, Tech Stack, Conventions, Key Patterns
    Review,
    /// Spec merge: Overview, Architecture, Key Patterns
    Archive,
}

impl ContextPhase {
    /// Get the section names to include for this phase
    pub fn sections(&self) -> Vec<&'static str> {
        match self {
            ContextPhase::Proposal | ContextPhase::Challenge | ContextPhase::Archive => {
                vec!["Overview", "Architecture", "Key Patterns"]
            }
            ContextPhase::Implement | ContextPhase::Review => {
                vec!["Overview", "Tech Stack", "Conventions", "Key Patterns"]
            }
        }
    }
}

/// Load specific sections from project.md based on the context phase
///
/// Parses agentd/project.md and extracts only the sections relevant to the phase.
/// Returns formatted markdown with the selected sections.
pub fn load_project_sections(phase: ContextPhase) -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let project_path = current_dir.join("agentd/project.md");

    // If project.md doesn't exist, return empty string
    if !project_path.exists() {
        return Ok(String::new());
    }

    let content = std::fs::read_to_string(&project_path)?;
    let sections_to_include = phase.sections();

    let mut result = String::new();
    let mut current_section: Option<&str> = None;
    let mut current_content = String::new();

    for line in content.lines() {
        // Check if this is a section header (## Level)
        if line.starts_with("## ") {
            // Save previous section if it was one we wanted
            if let Some(section_name) = current_section {
                if sections_to_include.contains(&section_name) && !current_content.trim().is_empty() {
                    result.push_str(&format!("## {}\n", section_name));
                    result.push_str(current_content.trim());
                    result.push_str("\n\n");
                }
            }

            // Start new section
            let section_name = line.trim_start_matches("## ").trim();
            current_section = sections_to_include.iter().find(|&&s| s == section_name).copied();
            current_content.clear();
        } else if current_section.is_some() {
            // Skip HTML comments
            if !line.trim().starts_with("<!--") && !line.trim().ends_with("-->") {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }
    }

    // Don't forget the last section
    if let Some(section_name) = current_section {
        if sections_to_include.contains(&section_name) && !current_content.trim().is_empty() {
            result.push_str(&format!("## {}\n", section_name));
            result.push_str(current_content.trim());
            result.push_str("\n\n");
        }
    }

    Ok(result.trim().to_string())
}

/// Generate GEMINI.md context file for a specific change
pub fn generate_gemini_context(change_dir: &Path, phase: ContextPhase) -> Result<()> {
    let project_structure = scan_project_structure()?;
    let project_context = load_project_sections(phase)?;

    let content = GEMINI_TEMPLATE
        .replace("{{PROJECT_STRUCTURE}}", &project_structure)
        .replace("{{PROJECT_CONTEXT}}", &project_context);

    let output_path = change_dir.join("GEMINI.md");
    std::fs::write(&output_path, content)?;

    Ok(())
}

/// Generate AGENTS.md context file for a specific change
pub fn generate_agents_context(change_dir: &Path, phase: ContextPhase) -> Result<()> {
    let project_structure = scan_project_structure()?;
    let project_context = load_project_sections(phase)?;

    let content = AGENTS_TEMPLATE
        .replace("{{PROJECT_STRUCTURE}}", &project_structure)
        .replace("{{PROJECT_CONTEXT}}", &project_context);

    let output_path = change_dir.join("AGENTS.md");
    std::fs::write(&output_path, content)?;

    Ok(())
}

/// Scan project structure and generate a tree representation
fn scan_project_structure() -> Result<String> {
    let current_dir = std::env::current_dir()?;
    let mut output = String::from("```\n");

    // Scan important directories
    let important_dirs = vec!["src", "agentd/specs", "agentd/changes"];

    for dir in important_dirs {
        let path = current_dir.join(dir);
        if path.exists() {
            output.push_str(&format!("{}:\n", dir));
            output.push_str(&scan_directory(&path, 2)?);
            output.push('\n');
        }
    }

    output.push_str("```");
    Ok(output)
}

/// Recursively scan a directory with depth limit
fn scan_directory(path: &Path, max_depth: usize) -> Result<String> {
    let mut output = String::new();
    let entries: Vec<_> = WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden files and common ignore patterns
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "dist"
        })
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let depth = entry.depth();
        if depth == 0 {
            continue;
        }

        let indent = "  ".repeat(depth);
        let name = entry.file_name().to_string_lossy();

        if entry.file_type().is_dir() {
            output.push_str(&format!("{}{}/\n", indent, name));
        } else {
            output.push_str(&format!("{}{}\n", indent, name));
        }
    }

    Ok(output)
}

/// Load a template file, checking for user override first, then falling back to embedded default.
///
/// - `name`: Template filename including extension (e.g., "proposal.md", "challenge.md")
/// - `project_root`: Project root directory to check for overrides
/// - `vars`: Key-value pairs for variable replacement (e.g., `change_id`, `iteration`)
///
/// Override path: `<project_root>/agentd/templates/<name>`
/// Variables use `{{key}}` syntax in templates.
pub fn load_template(name: &str, project_root: &Path, vars: &[(&str, &str)]) -> Result<String> {
    // Try to load user override first
    let override_path = project_root.join("agentd/templates").join(name);
    let content = if override_path.exists() {
        match std::fs::read_to_string(&override_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Shouldn't happen since we checked exists(), but handle gracefully
                get_embedded_template(name)?
            }
            Err(e) => {
                // Surface read errors (permissions, etc.) instead of silently falling back
                anyhow::bail!("Failed to read template override '{}': {}", override_path.display(), e);
            }
        }
    } else {
        get_embedded_template(name)?
    };

    // Replace variables: {{key}} -> value
    let mut result = content;
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key); // {{key}}
        result = result.replace(&placeholder, value);
    }

    Ok(result)
}

/// Get embedded template by name
fn get_embedded_template(name: &str) -> Result<String> {
    match name {
        "proposal.md" => Ok(PROPOSAL_SKELETON.to_string()),
        "tasks.md" => Ok(TASKS_SKELETON.to_string()),
        "spec.md" => Ok(SPEC_SKELETON.to_string()),
        "challenge.md" => Ok(CHALLENGE_SKELETON.to_string()),
        "review.md" => Ok(REVIEW_SKELETON.to_string()),
        "archive_review.md" => Ok(ARCHIVE_REVIEW_SKELETON.to_string()),
        _ => anyhow::bail!("Unknown template: {}", name),
    }
}

/// Derive project root from change directory
/// change_dir is typically: <project_root>/agentd/changes/<change_id>
fn derive_project_root(change_dir: &Path) -> Result<std::path::PathBuf> {
    // Walk up: change_dir -> changes -> agentd -> project_root
    change_dir
        .parent() // changes
        .and_then(|p| p.parent()) // agentd
        .and_then(|p| p.parent()) // project_root
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Could not derive project root from change_dir: {:?}", change_dir))
}

/// Create proposal skeleton files with structure but no content
/// This guides Gemini to fill in the right format while reducing token usage
pub fn create_proposal_skeleton(change_dir: &Path, change_id: &str) -> Result<()> {
    let project_root = derive_project_root(change_dir)?;
    let vars = [("change_id", change_id)];

    // Create proposal.md skeleton
    let proposal_content = load_template("proposal.md", &project_root, &vars)?;
    std::fs::write(change_dir.join("proposal.md"), proposal_content)?;

    // Create tasks.md skeleton
    let tasks_content = load_template("tasks.md", &project_root, &[])?;
    std::fs::write(change_dir.join("tasks.md"), tasks_content)?;

    // Create specs directory with a skeleton file
    let specs_dir = change_dir.join("specs");
    std::fs::create_dir_all(&specs_dir)?;

    let spec_content = load_template("spec.md", &project_root, &[])?;
    std::fs::write(specs_dir.join("_skeleton.md"), spec_content)?;

    Ok(())
}

/// Create challenge skeleton file to guide Codex review
/// This provides structure for consistent challenge reports
pub fn create_challenge_skeleton(change_dir: &Path, change_id: &str) -> Result<()> {
    let project_root = derive_project_root(change_dir)?;
    let vars = [("change_id", change_id)];

    let challenge_content = load_template("challenge.md", &project_root, &vars)?;
    std::fs::write(change_dir.join("CHALLENGE.md"), challenge_content)?;
    Ok(())
}

/// Create REVIEW.md skeleton for code review process
pub fn create_review_skeleton(change_dir: &Path, change_id: &str, iteration: u32) -> Result<()> {
    let project_root = derive_project_root(change_dir)?;
    let iteration_str = iteration.to_string();
    let vars = [("change_id", change_id), ("iteration", iteration_str.as_str())];

    let review_content = load_template("review.md", &project_root, &vars)?;
    std::fs::write(change_dir.join("REVIEW.md"), review_content)?;
    Ok(())
}

/// Create ARCHIVE_REVIEW.md skeleton for archive quality review
pub fn create_archive_review_skeleton(change_dir: &Path, change_id: &str, iteration: u32) -> Result<()> {
    let project_root = derive_project_root(change_dir)?;
    let iteration_str = iteration.to_string();
    let vars = [("change_id", change_id), ("iteration", iteration_str.as_str())];

    let archive_review_content = load_template("archive_review.md", &project_root, &vars)?;
    std::fs::write(change_dir.join("ARCHIVE_REVIEW.md"), archive_review_content)?;
    Ok(())
}

/// Clean up generated context files when archiving
pub fn cleanup_context_files(change_dir: &Path) -> Result<()> {
    let gemini_path = change_dir.join("GEMINI.md");
    let agents_path = change_dir.join("AGENTS.md");

    if gemini_path.exists() {
        std::fs::remove_file(gemini_path)?;
    }

    if agents_path.exists() {
        std::fs::remove_file(agents_path)?;
    }

    Ok(())
}

/// Conflict resolution strategy chosen by user
enum ConflictResolution {
    UseSuggested(String),
    Abort,
}

/// Resolves change-id conflicts by finding next available ID or prompting user
///
/// This function is called early in the proposal workflow (before calling LLMs)
/// to handle the case when a change directory already exists.
///
/// In interactive mode: Presents user with 3 options
/// In non-interactive mode: Auto-uses the suggested ID
pub fn resolve_change_id_conflict(change_id: &str, changes_dir: &Path) -> Result<String> {
    let change_dir = changes_dir.join(change_id);

    // No conflict - use original ID
    if !change_dir.exists() {
        return Ok(change_id.to_string());
    }

    // Conflict detected - find next available ID
    let suggested_id = find_next_available_id(change_id, changes_dir);
    let similar_changes = list_similar_changes(change_id, changes_dir);

    println!();
    println!("{}", "⚠️  Change already exists".yellow().bold());
    println!();

    // List similar existing changes
    if !similar_changes.is_empty() {
        println!("{}", "Existing changes:".bright_black());
        for change in &similar_changes {
            // Try to get creation time
            let change_path = changes_dir.join(change);
            if let Ok(metadata) = std::fs::metadata(&change_path) {
                if let Ok(created) = metadata.created() {
                    let datetime: chrono::DateTime<chrono::Local> = created.into();
                    println!(
                        "  • {}/ {}",
                        change,
                        format!("(created {})", datetime.format("%Y-%m-%d")).bright_black()
                    );
                } else {
                    println!("  • {}/", change);
                }
            } else {
                println!("  • {}/", change);
            }
        }
        println!();
    }

    // Try interactive prompt
    match prompt_conflict_resolution(change_id, &suggested_id, &change_dir) {
        Ok(resolution) => match resolution {
            ConflictResolution::UseSuggested(id) => {
                println!(
                    "{}",
                    format!("Using new ID: '{}'", id).green()
                );
                println!();
                Ok(id)
            }
            ConflictResolution::Abort => {
                anyhow::bail!("Operation aborted by user");
            }
        },
        Err(_) => {
            // Non-interactive mode or terminal not available
            // Auto-use suggested ID with warning
            println!(
                "{}",
                format!(
                    "(non-interactive mode: using new ID '{}')",
                    suggested_id
                )
                .bright_black()
            );
            println!();
            Ok(suggested_id)
        }
    }
}

/// Find next available change ID with numeric suffix
///
/// Given a base ID like "test-oauth", finds the next available numeric suffix:
/// - test-oauth exists -> test-oauth-2
/// - test-oauth, test-oauth-2 exist -> test-oauth-3
/// - test-oauth, test-oauth-5 exist -> test-oauth-6 (finds highest + 1)
fn find_next_available_id(base_id: &str, changes_dir: &Path) -> String {
    let mut highest = 1;

    // First, scan for any existing numbered versions to find the highest
    if let Ok(entries) = std::fs::read_dir(changes_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                // Check if this matches the pattern base_id-N
                if let Some(suffix) = name.strip_prefix(&format!("{}-", base_id)) {
                    if let Ok(num) = suffix.parse::<u32>() {
                        highest = highest.max(num);
                    }
                }
            }
        }
    }

    // Start from highest + 1
    let mut counter = highest + 1;

    // Find next available (in case there are gaps)
    loop {
        let candidate = format!("{}-{}", base_id, counter);
        if !changes_dir.join(&candidate).exists() {
            return candidate;
        }
        counter += 1;
    }
}

/// List existing changes with similar names
///
/// Returns a sorted list of change directories that start with the base_id.
/// For example, with base_id="test-oauth", returns:
/// ["test-oauth", "test-oauth-2", "test-oauth-3"]
fn list_similar_changes(base_id: &str, changes_dir: &Path) -> Vec<String> {
    let mut similar = Vec::new();

    if let Ok(entries) = std::fs::read_dir(changes_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Match exact or numbered pattern
                    if name == base_id || name.starts_with(&format!("{}-", base_id)) {
                        similar.push(name.to_string());
                    }
                }
            }
        }
    }

    similar.sort();
    similar
}

/// Interactive prompt for conflict resolution
///
/// Presents user with 2 options:
/// 1. Use suggested ID (recommended)
/// 2. Abort and manually handle
///
/// Returns Err if terminal is not available (non-interactive mode)
fn prompt_conflict_resolution(
    _original_id: &str,
    suggested_id: &str,
    _existing_path: &Path,
) -> Result<ConflictResolution> {
    let options = vec![
        format!("Use new ID '{}' (recommended)", suggested_id),
        "Abort (manually delete or use different ID)".to_string(),
    ];

    println!("{}", "What would you like to do?".cyan());

    let selection = Select::new()
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!("Terminal not available: {}", e))?;

    match selection {
        0 => Ok(ConflictResolution::UseSuggested(suggested_id.to_string())),
        1 => Ok(ConflictResolution::Abort),
        _ => Ok(ConflictResolution::Abort),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_template_embedded_default() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // No override exists, should use embedded default
        let result = load_template("proposal.md", project_root, &[]).unwrap();
        assert!(result.contains("# Change: {{change_id}}"));
    }

    #[test]
    fn test_load_template_with_variable_replacement() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let vars = [("change_id", "test-change")];
        let result = load_template("proposal.md", project_root, &vars).unwrap();
        assert!(result.contains("# Change: test-change"));
        assert!(!result.contains("{{change_id}}"));
    }

    #[test]
    fn test_load_template_override() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create override directory and file
        let templates_dir = project_root.join("agentd/templates");
        fs::create_dir_all(&templates_dir).unwrap();
        fs::write(
            templates_dir.join("proposal.md"),
            "# Custom Template\n\nChange: {{change_id}}",
        )
        .unwrap();

        let vars = [("change_id", "my-change")];
        let result = load_template("proposal.md", project_root, &vars).unwrap();
        assert!(result.contains("# Custom Template"));
        assert!(result.contains("Change: my-change"));
    }

    #[test]
    fn test_load_template_unknown_name() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        let result = load_template("unknown.md", project_root, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_project_root() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory structure
        let change_dir = project_root.join("agentd/changes/test-change");
        fs::create_dir_all(&change_dir).unwrap();

        let derived = derive_project_root(&change_dir).unwrap();
        assert_eq!(derived, project_root);
    }
}
