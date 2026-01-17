//! Code Analysis Strategy
//!
//! Analyzes source code files using AST parsing (tree-sitter) and generates
//! high-level technical specifications. Includes interactive clarification
//! and incremental update support.

use crate::fillback::ast::{AnalysisContext, AstAnalyzer, ModuleInfo, ParseError, SupportedLanguage};
use crate::fillback::graph::{DependencyGraph, GraphStats};
use crate::fillback::strategy::ImportStrategy;
use crate::Result;
use async_trait::async_trait;
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration for the code analysis strategy
#[derive(Debug, Clone)]
pub struct CodeStrategyConfig {
    /// Path to analyze (defaults to current directory)
    pub path: Option<String>,
    /// Specific module to analyze (optional filter)
    pub module: Option<String>,
    /// Force overwrite without confirmation
    pub force: bool,
    /// Output directory for specs (default: agentd/specs/)
    pub output_dir: Option<String>,
}

impl Default for CodeStrategyConfig {
    fn default() -> Self {
        Self {
            path: None,
            module: None,
            force: false,
            output_dir: None,
        }
    }
}

/// Code import strategy with AST-based analysis
///
/// Analyzes source code files using tree-sitter and generates
/// high-level technical designs and specifications.
pub struct CodeStrategy {
    config: CodeStrategyConfig,
}

impl CodeStrategy {
    pub fn new() -> Self {
        Self {
            config: CodeStrategyConfig::default(),
        }
    }

    pub fn with_config(config: CodeStrategyConfig) -> Self {
        Self { config }
    }

    /// Scan source directory and collect files for analysis
    fn scan_files(&self, source: &Path) -> Result<Vec<(String, String)>> {
        let mut files = Vec::new();
        let max_files = 500; // Higher limit since we're using AST
        let max_file_size = 100_000; // 100KB limit per file

        let walker = WalkBuilder::new(source)
            .standard_filters(true)
            .build();

        let mut skipped_count = 0;

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Check file size
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.len() > max_file_size as u64 {
                    skipped_count += 1;
                    continue;
                }
            }

            // Check if we support this language
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if SupportedLanguage::from_extension(ext).is_some() {
                    if let Ok(content) = fs::read_to_string(path) {
                        let relative_path = path
                            .strip_prefix(source)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string();
                        files.push((relative_path, content));

                        if files.len() >= max_files {
                            println!(
                                "{}",
                                format!("  Reached file limit ({}). Some files were skipped.", max_files)
                                    .yellow()
                            );
                            break;
                        }
                    }
                }
            }
        }

        if skipped_count > 0 {
            println!(
                "{}",
                format!("  Skipped {} files (too large)", skipped_count).bright_black()
            );
        }

        Ok(files)
    }

    /// Analyze codebase using AST parser
    pub fn analyze_codebase(&self, source: &Path) -> Result<(AnalysisContext, Vec<ParseError>)> {
        let mut analyzer = AstAnalyzer::new()?;
        let mut context = AnalysisContext::new();
        let mut parse_errors = Vec::new();

        let files = self.scan_files(source)?;

        if files.is_empty() {
            anyhow::bail!("No supported source files found in: {}", source.display());
        }

        println!(
            "{}",
            format!("  Analyzing {} files with tree-sitter...", files.len()).bright_black()
        );

        for (rel_path, content) in files {
            let full_path = source.join(&rel_path);

            match analyzer.parse_file(&full_path, &content) {
                Ok(module) => {
                    // Update language counts
                    let lang_name = module.language.display_name().to_string();
                    *context.language_counts.entry(lang_name).or_insert(0) += 1;

                    // Filter by module name if specified
                    if let Some(ref filter) = self.config.module {
                        if !module.name.contains(filter) {
                            continue;
                        }
                    }

                    context.modules.push(module);
                }
                Err(err) => {
                    context.skipped_files.push(rel_path.clone());
                    parse_errors.push(err);
                }
            }
        }

        if context.modules.is_empty() {
            if let Some(ref filter) = self.config.module {
                anyhow::bail!("No modules matching '{}' found", filter);
            } else {
                anyhow::bail!("Failed to parse any source files");
            }
        }

        Ok((context, parse_errors))
    }

    /// Display analysis summary
    pub fn display_summary(&self, context: &AnalysisContext, graph: &DependencyGraph) {
        let stats = GraphStats::from_graph(graph);

        println!();
        println!("{}", "Analysis Summary".cyan().bold());
        println!("{}", "----------------".bright_black());
        println!(
            "  Modules analyzed: {}",
            context.modules.len().to_string().green()
        );
        println!(
            "  Total symbols:    {}",
            context.total_symbols().to_string().green()
        );
        println!(
            "  External deps:    {}",
            stats.external_dependencies.to_string().yellow()
        );

        // Language breakdown
        if !context.language_counts.is_empty() {
            println!();
            println!("  {}", "Languages:".bright_black());
            for (lang, count) in &context.language_counts {
                println!("    {}: {} files", lang, count);
            }
        }

        // Most connected modules
        if !stats.most_connected_modules.is_empty() {
            println!();
            println!("  {}", "Most connected modules:".bright_black());
            for (name, count) in stats.most_connected_modules.iter().take(3) {
                println!("    {}: {} dependencies", name, count);
            }
        }

        // Skipped files
        if !context.skipped_files.is_empty() {
            println!();
            println!(
                "{}",
                format!("  Skipped {} files (parse errors)", context.skipped_files.len()).yellow()
            );
        }

        println!();
    }

    /// Display the dependency graph in compact form
    pub fn display_dependency_graph(&self, graph: &DependencyGraph) {
        println!("{}", "Dependency Graph (Mermaid)".cyan().bold());
        println!("{}", "-------------------------".bright_black());
        println!("{}", graph.to_mermaid_compact());
        println!();
    }

    /// Interactive clarification phase - asks questions to refine understanding
    pub fn run_clarification(&self, context: &AnalysisContext) -> Result<HashMap<String, String>> {
        let mut answers = HashMap::new();

        println!("{}", "Clarification Questions".cyan().bold());
        println!("{}", "-----------------------".bright_black());
        println!(
            "{}",
            "Please answer a few questions to improve specification quality:".bright_black()
        );
        println!();

        // Question 1: Main entry point
        let modules: Vec<&str> = context.modules.iter().map(|m| m.name.as_str()).collect();
        if !modules.is_empty() {
            let main_candidates: Vec<&str> = modules
                .iter()
                .filter(|m| {
                    m.contains("main") || m.contains("lib") || m.contains("app") || m.contains("index")
                })
                .copied()
                .collect();

            if !main_candidates.is_empty() {
                println!("Which module is the main entry point?");
                let selection = MultiSelect::new()
                    .items(&main_candidates)
                    .interact_opt()?;

                if let Some(indices) = selection {
                    let selected: Vec<String> = indices
                        .iter()
                        .map(|&i| main_candidates[i].to_string())
                        .collect();
                    answers.insert("entry_points".to_string(), selected.join(", "));
                }
            }
        }

        // Question 2: Public API modules
        let public_modules: Vec<&ModuleInfo> = context
            .modules
            .iter()
            .filter(|m| m.symbols.iter().any(|s| s.is_public))
            .collect();

        if !public_modules.is_empty() {
            println!();
            println!(
                "Found {} modules with public symbols. Which are part of the public API?",
                public_modules.len()
            );

            let module_names: Vec<&str> = public_modules.iter().map(|m| m.name.as_str()).collect();
            let selection = MultiSelect::new()
                .items(&module_names)
                .interact_opt()?;

            if let Some(indices) = selection {
                let selected: Vec<String> = indices
                    .iter()
                    .map(|&i| module_names[i].to_string())
                    .collect();
                answers.insert("public_api_modules".to_string(), selected.join(", "));
            }
        }

        // Question 3: Project description
        println!();
        let description: String = Input::new()
            .with_prompt("Brief project description (optional)")
            .allow_empty(true)
            .interact_text()?;

        if !description.is_empty() {
            answers.insert("project_description".to_string(), description);
        }

        // Question 4: Architecture style
        println!();
        let arch_styles = vec![
            "Monolithic",
            "Microservices",
            "Layered/Clean Architecture",
            "Event-Driven",
            "CLI Tool",
            "Library/SDK",
            "Other",
        ];

        println!("What architecture style best describes this project?");
        let selection = dialoguer::Select::new()
            .items(&arch_styles)
            .default(0)
            .interact_opt()?;

        if let Some(idx) = selection {
            answers.insert("architecture_style".to_string(), arch_styles[idx].to_string());
        }

        Ok(answers)
    }

    /// Check for existing specs and handle incremental updates
    pub fn check_existing_specs(&self, output_dir: &Path) -> Result<Vec<String>> {
        let mut existing_files = Vec::new();

        if output_dir.exists() {
            for entry in fs::read_dir(output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        existing_files.push(name.to_string());
                    }
                }
            }
        }

        Ok(existing_files)
    }

    /// Prompt for confirmation before overwriting existing specs
    pub fn confirm_overwrite(&self, existing_files: &[String]) -> Result<bool> {
        if self.config.force {
            return Ok(true);
        }

        if existing_files.is_empty() {
            return Ok(true);
        }

        println!();
        println!("{}", "Existing Specifications Found".yellow().bold());
        println!("{}", "-----------------------------".bright_black());
        for file in existing_files {
            println!("  - {}", file);
        }
        println!();

        let confirm = Confirm::new()
            .with_prompt("Overwrite existing specifications?")
            .default(false)
            .interact()?;

        Ok(confirm)
    }

    /// Generate specification files based on analysis
    pub fn generate_specs(
        &self,
        context: &AnalysisContext,
        graph: &DependencyGraph,
        output_dir: &Path,
        clarifications: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        fs::create_dir_all(output_dir)?;

        let mut created_files = Vec::new();

        // Generate dependency graph file
        let graph_content = graph.to_markdown("Analyzed Project");
        let graph_path = output_dir.join("_dependency-graph.md");
        fs::write(&graph_path, graph_content)?;
        created_files.push("_dependency-graph.md".to_string());

        // Generate overview spec
        let overview_content = self.generate_overview_spec(context, graph, clarifications);
        let overview_path = output_dir.join("_overview.md");
        fs::write(&overview_path, overview_content)?;
        created_files.push("_overview.md".to_string());

        // Generate spec for each major module
        for module in &context.modules {
            if module.symbols.is_empty() {
                continue;
            }

            let spec_content = self.generate_module_spec(module);
            let spec_name = format!("{}.md", module.name);
            let spec_path = output_dir.join(&spec_name);
            fs::write(&spec_path, spec_content)?;
            created_files.push(spec_name);
        }

        Ok(created_files)
    }

    /// Generate overview specification
    fn generate_overview_spec(
        &self,
        context: &AnalysisContext,
        graph: &DependencyGraph,
        clarifications: &HashMap<String, String>,
    ) -> String {
        let stats = GraphStats::from_graph(graph);

        let mut content = String::new();

        content.push_str("# Specification: Project Overview\n\n");

        content.push_str("## Summary\n\n");
        if let Some(desc) = clarifications.get("project_description") {
            content.push_str(&format!("{}\n\n", desc));
        } else {
            content.push_str("(Auto-generated from codebase analysis)\n\n");
        }

        // Architecture
        content.push_str("## Architecture\n\n");
        if let Some(style) = clarifications.get("architecture_style") {
            content.push_str(&format!("**Style**: {}\n\n", style));
        }

        content.push_str("### Module Structure\n\n");
        content.push_str("| Module | Symbols | Public | Language |\n");
        content.push_str("|--------|---------|--------|----------|\n");
        for module in &context.modules {
            let public_count = module.symbols.iter().filter(|s| s.is_public).count();
            content.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                module.name,
                module.symbols.len(),
                public_count,
                module.language.display_name()
            ));
        }

        // Entry points
        if let Some(entry_points) = clarifications.get("entry_points") {
            content.push_str("\n### Entry Points\n\n");
            for entry in entry_points.split(", ") {
                content.push_str(&format!("- `{}`\n", entry));
            }
        }

        // Public API
        if let Some(public_api) = clarifications.get("public_api_modules") {
            content.push_str("\n### Public API Modules\n\n");
            for module in public_api.split(", ") {
                content.push_str(&format!("- `{}`\n", module));
            }
        }

        // Dependencies
        content.push_str("\n## Dependencies\n\n");
        content.push_str(&format!(
            "- **Internal modules**: {}\n",
            stats.internal_modules
        ));
        content.push_str(&format!(
            "- **External dependencies**: {}\n",
            stats.external_dependencies
        ));
        content.push_str(&format!(
            "- **Avg dependencies/module**: {:.1}\n\n",
            stats.avg_dependencies_per_module
        ));

        let external_deps = graph.external_dependencies();
        if !external_deps.is_empty() {
            content.push_str("### External Dependencies\n\n");
            for dep in external_deps {
                content.push_str(&format!("- `{}`\n", dep.name));
            }
        }

        // Language breakdown
        if !context.language_counts.is_empty() {
            content.push_str("\n## Language Breakdown\n\n");
            for (lang, count) in &context.language_counts {
                content.push_str(&format!("- {}: {} files\n", lang, count));
            }
        }

        content
    }

    /// Generate module-specific specification
    fn generate_module_spec(&self, module: &ModuleInfo) -> String {
        let mut content = String::new();

        content.push_str(&format!("# Specification: {}\n\n", module.name));

        content.push_str("## Overview\n\n");
        content.push_str(&format!(
            "Module `{}` ({}) containing {} symbols.\n\n",
            module.name,
            module.language.display_name(),
            module.symbols.len()
        ));

        // Symbols table
        content.push_str("## Symbols\n\n");
        content.push_str("| Name | Kind | Visibility | Line |\n");
        content.push_str("|------|------|------------|------|\n");
        for symbol in &module.symbols {
            let visibility = if symbol.is_public { "public" } else { "private" };
            content.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                symbol.name, symbol.kind, visibility, symbol.line
            ));
        }

        // Function signatures
        let functions: Vec<_> = module
            .symbols
            .iter()
            .filter(|s| s.signature.is_some())
            .collect();
        if !functions.is_empty() {
            content.push_str("\n## Interfaces\n\n");
            content.push_str("```\n");
            for func in functions {
                if let Some(ref sig) = func.signature {
                    let doc = func.doc.as_deref().unwrap_or("");
                    if !doc.is_empty() {
                        content.push_str(&format!("// {}\n", doc));
                    }
                    content.push_str(&format!("{}\n\n", sig));
                }
            }
            content.push_str("```\n");
        }

        // Imports
        if !module.imports.is_empty() {
            content.push_str("\n## Dependencies\n\n");
            for import in &module.imports {
                let dep_type = if import.is_external {
                    "external"
                } else {
                    "internal"
                };
                content.push_str(&format!("- `{}` ({})\n", import.path, dep_type));
            }
        }

        content
    }

    /// Print summary of skipped files with errors
    pub fn print_parse_errors(&self, errors: &[ParseError]) {
        if errors.is_empty() {
            return;
        }

        println!();
        println!("{}", "Parse Errors".yellow().bold());
        println!("{}", "------------".bright_black());
        for error in errors.iter().take(10) {
            println!("  {}: {}", error.path.bright_black(), error.reason);
        }
        if errors.len() > 10 {
            println!("  ... and {} more", errors.len() - 10);
        }
        println!();
    }
}

impl Default for CodeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ImportStrategy for CodeStrategy {
    async fn execute(&self, source: &Path, _change_id: &str) -> Result<()> {
        println!();
        println!(
            "{}",
            format!("Scanning codebase at: {}", source.display()).cyan()
        );

        // Step 1: Analyze codebase with AST
        let (context, parse_errors) = self.analyze_codebase(source)?;

        // Step 2: Build dependency graph
        let graph = DependencyGraph::from_analysis(&context);

        // Step 3: Display analysis summary
        self.display_summary(&context, &graph);

        // Step 4: Display dependency graph
        self.display_dependency_graph(&graph);

        // Step 5: Print any parse errors
        self.print_parse_errors(&parse_errors);

        // Step 6: Run interactive clarification
        let clarifications = self.run_clarification(&context)?;

        // Step 7: Determine output directory
        let output_dir = if let Some(ref dir) = self.config.output_dir {
            std::path::PathBuf::from(dir)
        } else {
            std::env::current_dir()?.join("agentd/specs")
        };

        // Step 8: Check for existing specs
        let existing_specs = self.check_existing_specs(&output_dir)?;

        // Step 9: Confirm overwrite if needed
        if !self.confirm_overwrite(&existing_specs)? {
            println!("{}", "Cancelled by user.".yellow());
            return Ok(());
        }

        // Step 10: Generate specification files
        println!();
        println!("{}", "Generating specifications...".cyan());
        let created_files = self.generate_specs(&context, &graph, &output_dir, &clarifications)?;

        // Step 11: Summary
        println!();
        println!("{}", "Generated Files".green().bold());
        println!("{}", "---------------".bright_black());
        for file in &created_files {
            println!("  {}", output_dir.join(file).display());
        }

        println!();
        println!(
            "{}",
            format!(
                "Generated {} specification files in {}",
                created_files.len(),
                output_dir.display()
            )
            .green()
            .bold()
        );

        Ok(())
    }

    fn can_handle(&self, source: &Path) -> bool {
        source.is_dir()
    }

    fn name(&self) -> &'static str {
        "code"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(dir: &Path) {
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create main.rs
        fs::write(
            src_dir.join("main.rs"),
            r#"
use std::path::Path;

/// Main entry point
pub fn main() {
    println!("Hello, world!");
}

fn helper() -> i32 {
    42
}
"#,
        )
        .unwrap();

        // Create lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub mod utils;

pub struct Config {
    pub name: String,
}

pub fn init() -> Config {
    Config { name: "test".to_string() }
}
"#,
        )
        .unwrap();

        // Create utils.rs
        fs::write(
            src_dir.join("utils.rs"),
            r#"
use std::collections::HashMap;

pub fn format_string(s: &str) -> String {
    s.to_uppercase()
}

enum InternalEnum {
    A,
    B,
}
"#,
        )
        .unwrap();
    }

    #[test]
    fn test_scan_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::new();
        let files = strategy.scan_files(&temp_dir.path().join("src")).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|(path, _)| path.contains("main.rs")));
        assert!(files.iter().any(|(path, _)| path.contains("lib.rs")));
        assert!(files.iter().any(|(path, _)| path.contains("utils.rs")));
    }

    #[test]
    fn test_analyze_codebase() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::new();
        let (context, errors) = strategy
            .analyze_codebase(&temp_dir.path().join("src"))
            .unwrap();

        assert_eq!(context.modules.len(), 3);
        assert!(errors.is_empty() || errors.len() < context.modules.len());

        // Check symbols were extracted
        let total_symbols: usize = context.modules.iter().map(|m| m.symbols.len()).sum();
        assert!(total_symbols > 0);

        // Check language counts
        assert!(context.language_counts.contains_key("Rust"));
    }

    #[test]
    fn test_analyze_with_module_filter() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::with_config(CodeStrategyConfig {
            module: Some("main".to_string()),
            ..Default::default()
        });

        let (context, _) = strategy
            .analyze_codebase(&temp_dir.path().join("src"))
            .unwrap();

        assert_eq!(context.modules.len(), 1);
        assert_eq!(context.modules[0].name, "main");
    }

    #[test]
    fn test_check_existing_specs() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create some existing spec files
        fs::write(specs_dir.join("overview.md"), "# Overview").unwrap();
        fs::write(specs_dir.join("module_a.md"), "# Module A").unwrap();

        let strategy = CodeStrategy::new();
        let existing = strategy.check_existing_specs(&specs_dir).unwrap();

        assert_eq!(existing.len(), 2);
        assert!(existing.contains(&"overview.md".to_string()));
        assert!(existing.contains(&"module_a.md".to_string()));
    }

    #[test]
    fn test_generate_module_spec() {
        use crate::fillback::ast::{Import, Symbol, SymbolKind};

        let module = ModuleInfo {
            name: "test_module".to_string(),
            path: "src/test_module.rs".to_string(),
            language: SupportedLanguage::Rust,
            symbols: vec![
                Symbol {
                    name: "public_fn".to_string(),
                    kind: SymbolKind::Function,
                    signature: Some("public_fn(x: i32) -> String".to_string()),
                    doc: Some("A public function".to_string()),
                    line: 5,
                    is_public: true,
                },
                Symbol {
                    name: "TestStruct".to_string(),
                    kind: SymbolKind::Struct,
                    signature: None,
                    doc: None,
                    line: 10,
                    is_public: true,
                },
            ],
            imports: vec![Import {
                path: "std::collections".to_string(),
                items: vec![],
                is_external: true,
            }],
        };

        let strategy = CodeStrategy::new();
        let spec = strategy.generate_module_spec(&module);

        assert!(spec.contains("# Specification: test_module"));
        assert!(spec.contains("public_fn"));
        assert!(spec.contains("TestStruct"));
        assert!(spec.contains("std::collections"));
    }

    #[test]
    fn test_generate_specs_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::new();
        let (context, _) = strategy
            .analyze_codebase(&temp_dir.path().join("src"))
            .unwrap();
        let graph = DependencyGraph::from_analysis(&context);

        let output_dir = temp_dir.path().join("specs");
        let clarifications = HashMap::new();

        let created = strategy
            .generate_specs(&context, &graph, &output_dir, &clarifications)
            .unwrap();

        assert!(!created.is_empty());
        assert!(output_dir.join("_dependency-graph.md").exists());
        assert!(output_dir.join("_overview.md").exists());

        // At least one module spec should exist
        assert!(created.iter().any(|f| !f.starts_with('_')));
    }

    #[test]
    fn test_can_handle() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.rs");
        fs::write(&file, "fn main() {}").unwrap();

        let strategy = CodeStrategy::new();
        assert!(strategy.can_handle(temp_dir.path()));
        assert!(!strategy.can_handle(&file));
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let strategy = CodeStrategy::new();

        let result = strategy.analyze_codebase(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_force_overwrite() {
        let strategy = CodeStrategy::with_config(CodeStrategyConfig {
            force: true,
            ..Default::default()
        });

        let existing = vec!["file1.md".to_string(), "file2.md".to_string()];
        assert!(strategy.confirm_overwrite(&existing).unwrap());
    }
}
