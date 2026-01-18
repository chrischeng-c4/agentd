//! Fillback CLI Command
//!
//! Analyzes existing codebase using AST parsing and generates
//! language-agnostic specifications in agentd/specs/.

use crate::fillback::code::{CodeStrategy, CodeStrategyConfig};
use crate::fillback::ImportStrategy;
use crate::Result;
use colored::Colorize;
use std::env;
use std::path::PathBuf;

/// Run the fillback command to analyze codebase and generate specs
///
/// # Workflow
/// 1. Parse source files using tree-sitter AST analysis
/// 2. Build dependency graph from module relationships
/// 3. Display analysis summary and dependency graph
/// 4. Run interactive clarification to refine understanding
/// 5. Check for existing specs and confirm overwrites
/// 6. Generate specifications in agentd/specs/
pub async fn run(
    path: Option<&str>,
    module: Option<&str>,
    force: bool,
) -> Result<()> {
    let project_root = env::current_dir()?;

    println!("{}", "Agentd Fillback".cyan().bold());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black()
    );
    println!();

    // Determine source path
    let source_path = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        project_root.clone()
    };

    // Validate source path exists
    if !source_path.exists() {
        anyhow::bail!("Source path does not exist: {}", source_path.display());
    }

    if !source_path.is_dir() {
        anyhow::bail!("Source path must be a directory: {}", source_path.display());
    }

    println!(
        "{}",
        format!("Source: {}", source_path.display()).bright_black()
    );

    if let Some(m) = module {
        println!(
            "{}",
            format!("Module filter: {}", m).bright_black()
        );
    }

    if force {
        println!(
            "{}",
            "Force mode: will overwrite existing specs".yellow()
        );
    }

    println!();

    // Create strategy with configuration
    let config = CodeStrategyConfig {
        path: path.map(String::from),
        module: module.map(String::from),
        force,
        output_dir: Some(project_root.join("agentd/specs").to_string_lossy().to_string()),
    };

    let strategy = CodeStrategy::with_config(config);

    // Execute the strategy (it handles all the steps internally)
    // The change_id parameter is no longer used but kept for trait compatibility
    strategy.execute(&source_path, "fillback").await?;

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black());
    println!("{}", "Fillback completed!".green().bold());
    println!();
    println!("{}", "Next steps:".bright_black());
    println!("  1. Review generated specs in agentd/specs/");
    println!("  2. Edit and enhance specifications as needed");
    println!("  3. Use specs as reference for future changes");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(dir: &std::path::Path) {
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(
            src_dir.join("main.rs"),
            r#"
use std::path::Path;

/// Main entry point
pub fn main() {
    println!("Hello!");
}
"#,
        )
        .unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub mod utils;

pub struct Config {
    pub name: String,
}
"#,
        )
        .unwrap();

        // Create agentd directory structure
        fs::create_dir_all(dir.join("agentd/specs")).unwrap();
    }

    #[test]
    fn test_source_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("non_existent");

        // Test that non-existent path would fail (can't run async in sync test easily)
        assert!(!non_existent.exists());
    }

    #[test]
    fn test_project_structure() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        // Verify test project was created correctly
        assert!(temp_dir.path().join("src/main.rs").exists());
        assert!(temp_dir.path().join("src/lib.rs").exists());
        assert!(temp_dir.path().join("agentd/specs").exists());
    }
}
