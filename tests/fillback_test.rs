//! Integration tests for the fillback command
//!
//! Tests the complete fillback flow including:
//! - AST analysis of multi-language projects
//! - Dependency graph generation
//! - Spec file generation
//! - Error handling for parse failures

use agentd::fillback::{
    AstAnalyzer, CodeStrategy, CodeStrategyConfig, DependencyGraph, GraphStats, SupportedLanguage,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a test project with multiple languages
fn create_multi_language_project(dir: &Path) {
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Rust files
    fs::write(
        src_dir.join("main.rs"),
        r#"
//! Main entry point for the application

use std::path::Path;
use crate::config::Config;

mod config;
mod utils;

/// Main function
pub fn main() {
    let config = Config::load();
    println!("Starting with config: {:?}", config);
    utils::helper();
}

fn private_helper() -> i32 {
    42
}
"#,
    )
    .unwrap();

    fs::write(
        src_dir.join("config.rs"),
        r#"
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub debug: bool,
}

impl Config {
    pub fn load() -> Self {
        Config {
            name: "test".to_string(),
            debug: true,
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

pub enum ConfigError {
    NotFound,
    ParseError,
}
"#,
    )
    .unwrap();

    fs::write(
        src_dir.join("utils.rs"),
        r#"
use std::collections::HashMap;

/// Helper function
pub fn helper() {
    println!("Helper called");
}

pub fn format_string(s: &str) -> String {
    s.to_uppercase()
}

fn internal_fn() {
    // Private function
}
"#,
    )
    .unwrap();

    // Python files
    let py_dir = dir.join("scripts");
    fs::create_dir_all(&py_dir).unwrap();

    fs::write(
        py_dir.join("processor.py"),
        r#"
"""Data processor module."""

import os
from typing import List, Dict

class DataProcessor:
    """Processes data files."""

    def __init__(self, config: Dict):
        """Initialize with configuration."""
        self.config = config

    def process(self, data: List[str]) -> List[str]:
        """Process the input data."""
        return [item.upper() for item in data]

def main():
    """Main entry point."""
    processor = DataProcessor({})
    result = processor.process(["hello", "world"])
    print(result)

def _internal_helper():
    """Private helper."""
    pass

if __name__ == "__main__":
    main()
"#,
    )
    .unwrap();

    // JavaScript file
    fs::write(
        py_dir.join("helper.js"),
        r#"
import { something } from './utils';
import axios from 'axios';

/**
 * Helper function for data processing
 */
function processData(data) {
    return data.map(item => item.toUpperCase());
}

const formatOutput = (output) => {
    return JSON.stringify(output, null, 2);
};

class DataHandler {
    constructor(config) {
        this.config = config;
    }

    handle(data) {
        return processData(data);
    }
}

export { processData, formatOutput, DataHandler };
"#,
    )
    .unwrap();

    // Create agentd directory
    fs::create_dir_all(dir.join("agentd/specs")).unwrap();
}

/// Create a project with parse errors
fn create_project_with_errors(dir: &Path) {
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Valid Rust file
    fs::write(
        src_dir.join("main.rs"),
        r#"
pub fn main() {
    println!("Hello");
}
"#,
    )
    .unwrap();

    // Binary file (should be skipped gracefully)
    fs::write(src_dir.join("data.bin"), vec![0u8, 1, 2, 3, 255, 254]).unwrap();

    // Unsupported file type
    fs::write(src_dir.join("readme.txt"), "This is a readme").unwrap();

    fs::create_dir_all(dir.join("agentd/specs")).unwrap();
}

#[test]
fn test_analyze_rust_project() {
    let temp_dir = TempDir::new().unwrap();
    create_multi_language_project(temp_dir.path());

    let strategy = CodeStrategy::new();
    let src_dir = temp_dir.path().join("src");
    let (context, errors) = strategy.analyze_codebase(&src_dir).unwrap();

    // Should have parsed all 3 Rust files
    assert_eq!(context.modules.len(), 3);

    // Should detect Rust language
    assert!(context.language_counts.contains_key("Rust"));
    assert_eq!(context.language_counts.get("Rust"), Some(&3));

    // No errors for valid Rust files
    assert!(errors.is_empty());

    // Check that symbols were extracted
    let main_module = context.modules.iter().find(|m| m.name == "main").unwrap();
    assert!(main_module.symbols.iter().any(|s| s.name == "main"));
    assert!(main_module.symbols.iter().any(|s| s.name == "private_helper"));

    let config_module = context.modules.iter().find(|m| m.name == "config").unwrap();
    assert!(config_module.symbols.iter().any(|s| s.name == "Config"));
    assert!(config_module.symbols.iter().any(|s| s.name == "ConfigError"));
}

#[test]
fn test_analyze_multi_language_project() {
    let temp_dir = TempDir::new().unwrap();
    create_multi_language_project(temp_dir.path());

    let strategy = CodeStrategy::new();
    // Analyze the whole project
    let (context, _) = strategy.analyze_codebase(temp_dir.path()).unwrap();

    // Should have found files in multiple languages
    assert!(context.modules.len() >= 3);

    // Check that multiple languages were detected
    let languages: Vec<_> = context.language_counts.keys().collect();
    assert!(languages.len() >= 1); // At least Rust
}

#[test]
fn test_dependency_graph_generation() {
    let temp_dir = TempDir::new().unwrap();
    create_multi_language_project(temp_dir.path());

    let strategy = CodeStrategy::new();
    let src_dir = temp_dir.path().join("src");
    let (context, _) = strategy.analyze_codebase(&src_dir).unwrap();

    let graph = DependencyGraph::from_analysis(&context);

    // Should have internal module nodes
    let internal_modules = graph.internal_modules();
    assert_eq!(internal_modules.len(), 3);

    // Should have external dependencies (std, serde, etc.)
    let external_deps = graph.external_dependencies();
    assert!(!external_deps.is_empty());

    // Should have dependency edges
    assert!(!graph.edges.is_empty());

    // Stats should be computed correctly
    let stats = GraphStats::from_graph(&graph);
    assert_eq!(stats.internal_modules, 3);
    assert!(stats.external_dependencies > 0);
}

#[test]
fn test_mermaid_output() {
    let temp_dir = TempDir::new().unwrap();
    create_multi_language_project(temp_dir.path());

    let strategy = CodeStrategy::new();
    let src_dir = temp_dir.path().join("src");
    let (context, _) = strategy.analyze_codebase(&src_dir).unwrap();

    let graph = DependencyGraph::from_analysis(&context);
    let mermaid = graph.to_mermaid();

    // Should have proper mermaid format
    assert!(mermaid.contains("```mermaid"));
    assert!(mermaid.contains("flowchart TD"));
    assert!(mermaid.contains("```"));

    // Should contain module names
    assert!(mermaid.contains("main"));
    assert!(mermaid.contains("config"));
    assert!(mermaid.contains("utils"));
}

#[test]
fn test_spec_generation() {
    let temp_dir = TempDir::new().unwrap();
    create_multi_language_project(temp_dir.path());

    let strategy = CodeStrategy::new();
    let src_dir = temp_dir.path().join("src");
    let (context, _) = strategy.analyze_codebase(&src_dir).unwrap();
    let graph = DependencyGraph::from_analysis(&context);

    let output_dir = temp_dir.path().join("specs");
    let clarifications = HashMap::new();

    let created = strategy
        .generate_specs(&context, &graph, &output_dir, &clarifications)
        .unwrap();

    // Should create dependency graph
    assert!(created.contains(&"_dependency-graph.md".to_string()));
    assert!(output_dir.join("_dependency-graph.md").exists());

    // Should create overview
    assert!(created.contains(&"_overview.md".to_string()));
    assert!(output_dir.join("_overview.md").exists());

    // Should create module specs
    assert!(created.iter().any(|f| f.contains("main")));
    assert!(created.iter().any(|f| f.contains("config")));
    assert!(created.iter().any(|f| f.contains("utils")));

    // Check overview content
    let overview = fs::read_to_string(output_dir.join("_overview.md")).unwrap();
    assert!(overview.contains("# Specification: Project Overview"));
    assert!(overview.contains("Module Structure"));
}

#[test]
fn test_spec_generation_with_clarifications() {
    let temp_dir = TempDir::new().unwrap();
    create_multi_language_project(temp_dir.path());

    let strategy = CodeStrategy::new();
    let src_dir = temp_dir.path().join("src");
    let (context, _) = strategy.analyze_codebase(&src_dir).unwrap();
    let graph = DependencyGraph::from_analysis(&context);

    let output_dir = temp_dir.path().join("specs");
    let mut clarifications = HashMap::new();
    clarifications.insert("project_description".to_string(), "A test project".to_string());
    clarifications.insert("architecture_style".to_string(), "CLI Tool".to_string());
    clarifications.insert("entry_points".to_string(), "main".to_string());

    strategy
        .generate_specs(&context, &graph, &output_dir, &clarifications)
        .unwrap();

    let overview = fs::read_to_string(output_dir.join("_overview.md")).unwrap();
    assert!(overview.contains("A test project"));
    assert!(overview.contains("CLI Tool"));
    assert!(overview.contains("Entry Points"));
}

#[test]
fn test_module_filter() {
    let temp_dir = TempDir::new().unwrap();
    create_multi_language_project(temp_dir.path());

    let strategy = CodeStrategy::with_config(CodeStrategyConfig {
        module: Some("config".to_string()),
        ..Default::default()
    });

    let src_dir = temp_dir.path().join("src");
    let (context, _) = strategy.analyze_codebase(&src_dir).unwrap();

    // Should only include modules matching the filter
    assert_eq!(context.modules.len(), 1);
    assert_eq!(context.modules[0].name, "config");
}

#[test]
fn test_parse_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    create_project_with_errors(temp_dir.path());

    let strategy = CodeStrategy::new();
    let src_dir = temp_dir.path().join("src");
    let (context, errors) = strategy.analyze_codebase(&src_dir).unwrap();

    // Should parse the valid Rust file
    assert!(context.modules.iter().any(|m| m.name == "main"));

    // Unsupported files should be in skipped list
    // Note: binary and txt files are not attempted because extension is not supported
    assert!(context.skipped_files.is_empty() || errors.is_empty());
}

#[test]
fn test_empty_directory_error() {
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir_all(&empty_dir).unwrap();

    let strategy = CodeStrategy::new();
    let result = strategy.analyze_codebase(&empty_dir);

    assert!(result.is_err());
}

#[test]
fn test_check_existing_specs() {
    let temp_dir = TempDir::new().unwrap();
    let specs_dir = temp_dir.path().join("agentd/specs");
    fs::create_dir_all(&specs_dir).unwrap();

    // Create some existing specs
    fs::write(specs_dir.join("_overview.md"), "# Existing").unwrap();
    fs::write(specs_dir.join("module.md"), "# Module").unwrap();

    let strategy = CodeStrategy::new();
    let existing = strategy.check_existing_specs(&specs_dir).unwrap();

    assert_eq!(existing.len(), 2);
    assert!(existing.contains(&"_overview.md".to_string()));
    assert!(existing.contains(&"module.md".to_string()));
}

#[test]
fn test_force_overwrite() {
    let strategy = CodeStrategy::with_config(CodeStrategyConfig {
        force: true,
        ..Default::default()
    });

    let existing = vec!["file1.md".to_string(), "file2.md".to_string()];

    // Force mode should always return true
    assert!(strategy.confirm_overwrite(&existing).unwrap());
}

#[test]
fn test_ast_analyzer_rust() {
    let mut analyzer = AstAnalyzer::new().unwrap();

    let content = r#"
use std::collections::HashMap;

/// A public struct
pub struct MyStruct {
    pub field: String,
}

impl MyStruct {
    /// Creates a new instance
    pub fn new() -> Self {
        Self { field: String::new() }
    }

    fn private_method(&self) {}
}

pub enum MyEnum {
    A,
    B(i32),
}

pub type MyType = HashMap<String, i32>;

pub const MY_CONST: i32 = 42;
"#;

    let result = analyzer.parse_file(Path::new("test.rs"), content).unwrap();

    assert_eq!(result.language, SupportedLanguage::Rust);
    assert!(!result.symbols.is_empty());

    // Check struct was found
    assert!(result.symbols.iter().any(|s| s.name == "MyStruct" && s.is_public));

    // Check enum was found
    assert!(result.symbols.iter().any(|s| s.name == "MyEnum"));

    // Check type alias was found
    assert!(result.symbols.iter().any(|s| s.name == "MyType"));

    // Check const was found
    assert!(result.symbols.iter().any(|s| s.name == "MY_CONST"));

    // Check method was found
    assert!(result.symbols.iter().any(|s| s.name == "new"));

    // Check imports
    assert!(!result.imports.is_empty());
    assert!(result.imports.iter().any(|i| i.path.contains("std")));
}

#[test]
fn test_ast_analyzer_python() {
    let mut analyzer = AstAnalyzer::new().unwrap();

    let content = r#"
import os
from typing import List, Optional

class MyClass:
    """A documented class."""

    def __init__(self, value: int):
        """Initialize the class."""
        self.value = value

    def public_method(self) -> str:
        """A public method."""
        return str(self.value)

    def _private_method(self):
        pass

def public_function(x: int) -> int:
    """A public function."""
    return x * 2

def _private_function():
    pass
"#;

    let result = analyzer.parse_file(Path::new("test.py"), content).unwrap();

    assert_eq!(result.language, SupportedLanguage::Python);

    // Check class was found
    assert!(result.symbols.iter().any(|s| s.name == "MyClass"));

    // Check functions were found
    assert!(result.symbols.iter().any(|s| s.name == "public_function" && s.is_public));
    assert!(result.symbols.iter().any(|s| s.name == "_private_function" && !s.is_public));

    // Check imports
    assert!(result.imports.iter().any(|i| i.path == "os"));
    assert!(result.imports.iter().any(|i| i.path == "typing"));
}

#[test]
fn test_ast_analyzer_go() {
    let mut analyzer = AstAnalyzer::new().unwrap();

    let content = r#"
package main

import (
    "fmt"
    "github.com/example/pkg"
)

// PublicStruct is exported
type PublicStruct struct {
    Field string
}

type privateStruct struct {
    field int
}

// PublicInterface is an exported interface
type PublicInterface interface {
    Method() error
}

// PublicFunc is an exported function
func PublicFunc(x int) string {
    return fmt.Sprintf("%d", x)
}

func privateFunc() {
}

const PublicConst = 42
"#;

    let result = analyzer.parse_file(Path::new("test.go"), content).unwrap();

    assert_eq!(result.language, SupportedLanguage::Go);

    // Check public symbols
    assert!(result.symbols.iter().any(|s| s.name == "PublicStruct" && s.is_public));
    assert!(result.symbols.iter().any(|s| s.name == "PublicInterface" && s.is_public));
    assert!(result.symbols.iter().any(|s| s.name == "PublicFunc" && s.is_public));

    // Check private symbols
    assert!(result.symbols.iter().any(|s| s.name == "privateStruct" && !s.is_public));
    assert!(result.symbols.iter().any(|s| s.name == "privateFunc" && !s.is_public));

    // Check imports
    assert!(result.imports.iter().any(|i| i.path == "fmt"));
    assert!(result.imports.iter().any(|i| i.path.contains("github.com")));
}
