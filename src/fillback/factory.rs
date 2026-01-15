use crate::fillback::code::CodeStrategy;
use crate::fillback::openspec::OpenSpecStrategy;
use crate::fillback::speckit::SpeckitStrategy;
use crate::fillback::strategy::ImportStrategy;
use crate::Result;
use colored::Colorize;
use std::path::Path;

/// Factory for creating import strategy instances
///
/// Provides a centralized way to create strategies based on user input
/// or auto-detection logic.
pub struct StrategyFactory;

impl StrategyFactory {
    /// Create a strategy based on the strategy type and source path
    ///
    /// # Arguments
    /// * `strategy_type` - The strategy type ("auto", "openspec", "speckit", "code")
    /// * `source` - Path to the source to import from
    ///
    /// # Returns
    /// A boxed ImportStrategy instance
    ///
    /// # Errors
    /// Returns an error if the strategy type is invalid or auto-detection fails
    pub fn create(
        strategy_type: &str,
        source: &Path,
    ) -> Result<Box<dyn ImportStrategy>> {
        match strategy_type {
            "openspec" => Ok(Box::new(OpenSpecStrategy::new())),
            "speckit" => Ok(Box::new(SpeckitStrategy::new())),
            "code" => Ok(Box::new(CodeStrategy::new())),
            "auto" => Self::auto_detect(source),
            _ => {
                anyhow::bail!(
                    "Invalid strategy: '{}'. Supported strategies: auto, openspec, speckit, code",
                    strategy_type
                );
            }
        }
    }

    /// Auto-detect the appropriate strategy for the given source
    ///
    /// Tries each strategy's `can_handle` method in order:
    /// 1. OpenSpec (YAML/JSON files)
    /// 2. Speckit (Markdown files)
    /// 3. Code (directories)
    ///
    /// # Arguments
    /// * `source` - Path to analyze
    ///
    /// # Returns
    /// The first strategy that can handle the source
    ///
    /// # Errors
    /// Returns an error if no strategy can handle the source
    fn auto_detect(source: &Path) -> Result<Box<dyn ImportStrategy>> {
        // Try strategies in order of specificity
        let strategies: Vec<Box<dyn ImportStrategy>> = vec![
            Box::new(OpenSpecStrategy::new()),
            Box::new(SpeckitStrategy::new()),
            Box::new(CodeStrategy::new()),
        ];

        for strategy in strategies {
            if strategy.can_handle(source) {
                let name = strategy.name();
                println!(
                    "{}",
                    format!("🔍 Auto-detected strategy: {}", name).cyan()
                );
                return Ok(strategy);
            }
        }

        anyhow::bail!(
            "Could not auto-detect strategy for: {}. Please specify --strategy explicitly.",
            source.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_openspec_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_file = temp_dir.path().join("spec.yaml");
        fs::write(&yaml_file, "name: Test\ndescription: Test spec").unwrap();

        let strategy = StrategyFactory::create("openspec", &yaml_file).unwrap();
        assert_eq!(strategy.name(), "openspec");
    }

    #[test]
    fn test_create_speckit_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("spec.md");
        fs::write(&md_file, "# Test").unwrap();

        let strategy = StrategyFactory::create("speckit", &md_file).unwrap();
        assert_eq!(strategy.name(), "speckit");
    }

    #[test]
    fn test_create_code_strategy() {
        let temp_dir = TempDir::new().unwrap();

        let strategy = StrategyFactory::create("code", temp_dir.path()).unwrap();
        assert_eq!(strategy.name(), "code");
    }

    #[test]
    fn test_create_invalid_strategy() {
        let temp_dir = TempDir::new().unwrap();
        let result = StrategyFactory::create("invalid", temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_detect_openspec() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_file = temp_dir.path().join("spec.yaml");
        fs::write(&yaml_file, "name: Test\ndescription: Test spec").unwrap();

        let strategy = StrategyFactory::create("auto", &yaml_file).unwrap();
        assert_eq!(strategy.name(), "openspec");
    }

    #[test]
    fn test_auto_detect_code() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        let strategy = StrategyFactory::create("auto", temp_dir.path()).unwrap();
        assert_eq!(strategy.name(), "code");
    }

    #[test]
    fn test_auto_detect_failure() {
        let temp_dir = TempDir::new().unwrap();
        let txt_file = temp_dir.path().join("test.txt");
        fs::write(&txt_file, "Not a spec file").unwrap();

        let result = StrategyFactory::create("auto", &txt_file);
        assert!(result.is_err());
    }
}
