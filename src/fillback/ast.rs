//! AST Analysis Module
//!
//! Provides tree-sitter based parsing for supported languages.
//! Extracts modules, functions, structs, and imports from source files.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

/// Supported programming languages for AST analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
}

impl SupportedLanguage {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            _ => None,
        }
    }

    /// Get the tree-sitter language for this language
    fn tree_sitter_language(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
        }
    }

    /// Get the display name for this language
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
        }
    }
}

/// Kind of symbol extracted from source code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Interface,
    Class,
    Module,
    Constant,
    Type,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Interface => write!(f, "interface"),
            Self::Class => write!(f, "class"),
            Self::Module => write!(f, "module"),
            Self::Constant => write!(f, "constant"),
            Self::Type => write!(f, "type"),
        }
    }
}

/// A symbol extracted from source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub signature: Option<String>,
    pub doc: Option<String>,
    pub line: usize,
    pub is_public: bool,
}

/// An import/dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub items: Vec<String>,
    pub is_external: bool,
}

/// Parsed module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub language: SupportedLanguage,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
}

/// Parse error information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub path: String,
    pub reason: String,
}

/// Result of analyzing a codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    pub modules: Vec<ModuleInfo>,
    pub skipped_files: Vec<String>,
    pub language_counts: HashMap<String, usize>,
}

impl AnalysisContext {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            skipped_files: Vec::new(),
            language_counts: HashMap::new(),
        }
    }

    /// Get total symbol count across all modules
    pub fn total_symbols(&self) -> usize {
        self.modules.iter().map(|m| m.symbols.len()).sum()
    }

    /// Get all unique external dependencies
    pub fn external_dependencies(&self) -> Vec<String> {
        let mut deps: Vec<String> = self
            .modules
            .iter()
            .flat_map(|m| m.imports.iter())
            .filter(|i| i.is_external)
            .map(|i| i.path.clone())
            .collect();
        deps.sort();
        deps.dedup();
        deps
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

/// AST Analyzer using tree-sitter
pub struct AstAnalyzer {
    parsers: HashMap<SupportedLanguage, Parser>,
}

impl AstAnalyzer {
    /// Create a new AST analyzer with parsers for all supported languages
    pub fn new() -> Result<Self> {
        let mut parsers = HashMap::new();

        for lang in [
            SupportedLanguage::Rust,
            SupportedLanguage::Python,
            SupportedLanguage::JavaScript,
            SupportedLanguage::TypeScript,
            SupportedLanguage::Go,
        ] {
            let mut parser = Parser::new();
            parser.set_language(&lang.tree_sitter_language())?;
            parsers.insert(lang, parser);
        }

        Ok(Self { parsers })
    }

    /// Parse a single file and extract module information
    pub fn parse_file(
        &mut self,
        path: &Path,
        content: &str,
    ) -> std::result::Result<ModuleInfo, ParseError> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let language = SupportedLanguage::from_extension(ext).ok_or_else(|| ParseError {
            path: path.display().to_string(),
            reason: format!("Unsupported file extension: {}", ext),
        })?;

        let parser = self.parsers.get_mut(&language).ok_or_else(|| ParseError {
            path: path.display().to_string(),
            reason: format!("No parser for language: {:?}", language),
        })?;

        let tree = parser.parse(content, None).ok_or_else(|| ParseError {
            path: path.display().to_string(),
            reason: "Failed to parse file".to_string(),
        })?;

        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let (symbols, imports) = self.extract_symbols_and_imports(&tree, content, language);

        Ok(ModuleInfo {
            name: module_name,
            path: path.display().to_string(),
            language,
            symbols,
            imports,
        })
    }

    /// Extract symbols and imports from a parsed tree
    fn extract_symbols_and_imports(
        &self,
        tree: &Tree,
        source: &str,
        language: SupportedLanguage,
    ) -> (Vec<Symbol>, Vec<Import>) {
        let mut symbols = Vec::new();
        let mut imports = Vec::new();

        let root = tree.root_node();
        let mut cursor = root.walk();

        // Walk through top-level nodes
        for node in root.children(&mut cursor) {
            match language {
                SupportedLanguage::Rust => {
                    self.extract_rust_node(&node, source, &mut symbols, &mut imports);
                }
                SupportedLanguage::Python => {
                    self.extract_python_node(&node, source, &mut symbols, &mut imports);
                }
                SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                    self.extract_js_node(&node, source, &mut symbols, &mut imports);
                }
                SupportedLanguage::Go => {
                    self.extract_go_node(&node, source, &mut symbols, &mut imports);
                }
            }
        }

        (symbols, imports)
    }

    /// Extract symbols and imports from Rust AST nodes
    fn extract_rust_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        let kind = node.kind();

        match kind {
            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);
                    let signature = self.extract_rust_function_signature(node, source);
                    let doc = self.extract_rust_doc_comment(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "struct_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);
                    let doc = self.extract_rust_doc_comment(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Struct,
                        signature: None,
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "enum_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);
                    let doc = self.extract_rust_doc_comment(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Enum,
                        signature: None,
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "impl_item" => {
                // Extract methods from impl blocks
                self.extract_rust_impl_methods(node, source, symbols);
            }
            "use_declaration" => {
                let import = self.extract_rust_use(node, source);
                if let Some(import) = import {
                    imports.push(import);
                }
            }
            "mod_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Module,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "type_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Type,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "const_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Constant,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            _ => {}
        }
    }

    /// Extract methods from Rust impl block
    fn extract_rust_impl_methods(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                let mut inner_cursor = child.walk();
                for item in child.children(&mut inner_cursor) {
                    if item.kind() == "function_item" {
                        if let Some(name_node) = item.child_by_field_name("name") {
                            let name = self.node_text(&name_node, source);
                            let is_public = self.has_visibility_modifier(&item, source);
                            let signature = self.extract_rust_function_signature(&item, source);
                            let doc = self.extract_rust_doc_comment(&item, source);

                            symbols.push(Symbol {
                                name,
                                kind: SymbolKind::Function,
                                signature: Some(signature),
                                doc,
                                line: item.start_position().row + 1,
                                is_public,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Extract Rust function signature
    fn extract_rust_function_signature(&self, node: &tree_sitter::Node, source: &str) -> String {
        let mut signature = String::new();

        // Get function name
        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        // Get parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        // Get return type
        if let Some(return_node) = node.child_by_field_name("return_type") {
            signature.push_str(" -> ");
            signature.push_str(&self.node_text(&return_node, source));
        }

        signature
    }

    /// Extract Rust doc comment (/// or //!)
    fn extract_rust_doc_comment(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Look for preceding line comments that are doc comments
        let start_row = node.start_position().row;
        if start_row == 0 {
            return None;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut doc_lines = Vec::new();

        // Look backwards from the node for doc comments
        let mut row = start_row.saturating_sub(1);
        while row < lines.len() {
            let line = lines[row].trim();
            if line.starts_with("///") {
                doc_lines.insert(0, line.trim_start_matches("///").trim());
            } else if line.starts_with("//!") {
                doc_lines.insert(0, line.trim_start_matches("//!").trim());
            } else if !line.is_empty() && !line.starts_with("//") {
                break;
            }
            if row == 0 {
                break;
            }
            row -= 1;
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// Check if node has visibility modifier (pub)
    fn has_visibility_modifier(&self, node: &tree_sitter::Node, source: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                return true;
            }
            // Also check if line starts with pub
            let text = self.node_text(node, source);
            if text.starts_with("pub ") || text.starts_with("pub(") {
                return true;
            }
        }
        false
    }

    /// Extract Rust use statement
    fn extract_rust_use(&self, node: &tree_sitter::Node, source: &str) -> Option<Import> {
        let text = self.node_text(node, source);

        // Parse use statement (simplified)
        let path = text
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .to_string();

        // Determine if external (doesn't start with crate::, self::, super::)
        let is_external = !path.starts_with("crate::")
            && !path.starts_with("self::")
            && !path.starts_with("super::");

        // Extract the base path (before any braces or ::*)
        let base_path = path
            .split("::{")
            .next()
            .unwrap_or(&path)
            .split("::*")
            .next()
            .unwrap_or(&path)
            .to_string();

        Some(Import {
            path: base_path,
            items: vec![],
            is_external,
        })
    }

    /// Extract symbols and imports from Python AST nodes
    fn extract_python_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        let kind = node.kind();

        match kind {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = !name.starts_with('_');
                    let signature = self.extract_python_function_signature(node, source);
                    let doc = self.extract_python_docstring(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "class_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = !name.starts_with('_');
                    let doc = self.extract_python_docstring(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        signature: None,
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "import_statement" | "import_from_statement" => {
                let import = self.extract_python_import(node, source);
                if let Some(import) = import {
                    imports.push(import);
                }
            }
            _ => {}
        }
    }

    /// Extract Python function signature
    fn extract_python_function_signature(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> String {
        let mut signature = String::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        if let Some(return_node) = node.child_by_field_name("return_type") {
            signature.push_str(" -> ");
            signature.push_str(&self.node_text(&return_node, source));
        }

        signature
    }

    /// Extract Python docstring
    fn extract_python_docstring(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Look for string expression as first statement in body
        let body = node.child_by_field_name("body")?;
        let mut cursor = body.walk();
        // Only check the first child for docstring
        let first_child = body.children(&mut cursor).next()?;

        if first_child.kind() == "expression_statement" {
            let mut inner_cursor = first_child.walk();
            for expr in first_child.children(&mut inner_cursor) {
                if expr.kind() == "string" {
                    let text = self.node_text(&expr, source);
                    // Remove quotes
                    let doc = text
                        .trim_start_matches("\"\"\"")
                        .trim_end_matches("\"\"\"")
                        .trim_start_matches("'''")
                        .trim_end_matches("'''")
                        .trim_start_matches('"')
                        .trim_end_matches('"')
                        .trim()
                        .to_string();
                    return Some(doc);
                }
            }
        }
        None
    }

    /// Extract Python import
    fn extract_python_import(&self, node: &tree_sitter::Node, source: &str) -> Option<Import> {
        let text = self.node_text(node, source);

        // Parse import statement (simplified)
        let (path, items) = if text.starts_with("from ") {
            let parts: Vec<&str> = text.split(" import ").collect();
            let path = parts
                .first()?
                .trim_start_matches("from ")
                .trim()
                .to_string();
            let items: Vec<String> = parts
                .get(1)
                .map(|s| s.split(',').map(|i| i.trim().to_string()).collect())
                .unwrap_or_default();
            (path, items)
        } else {
            let path = text.trim_start_matches("import ").trim().to_string();
            (path, vec![])
        };

        // Determine if external (doesn't start with .)
        let is_external = !path.starts_with('.');

        Some(Import {
            path,
            items,
            is_external,
        })
    }

    /// Extract symbols and imports from JavaScript/TypeScript AST nodes
    fn extract_js_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        let kind = node.kind();

        match kind {
            "function_declaration" | "generator_function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let signature = self.extract_js_function_signature(node, source);
                    let doc = self.extract_jsdoc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public: true, // JS exports determine visibility
                    });
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let doc = self.extract_jsdoc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        signature: None,
                        doc,
                        line: node.start_position().row + 1,
                        is_public: true,
                    });
                }
            }
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Interface,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public: true,
                    });
                }
            }
            "type_alias_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Type,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public: true,
                    });
                }
            }
            "lexical_declaration" => {
                // const/let declarations
                self.extract_js_variable_declaration(node, source, symbols);
            }
            "import_statement" => {
                let import = self.extract_js_import(node, source);
                if let Some(import) = import {
                    imports.push(import);
                }
            }
            "export_statement" => {
                // Handle export default function, export const, etc.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_js_node(&child, source, symbols, imports);
                }
            }
            _ => {}
        }
    }

    /// Extract JS function signature
    fn extract_js_function_signature(&self, node: &tree_sitter::Node, source: &str) -> String {
        let mut signature = String::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        signature
    }

    /// Extract JS variable declaration
    fn extract_js_variable_declaration(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);

                    // Check if it's a function expression
                    if let Some(value_node) = child.child_by_field_name("value") {
                        let kind = match value_node.kind() {
                            "arrow_function" | "function" => SymbolKind::Function,
                            _ => SymbolKind::Constant,
                        };

                        symbols.push(Symbol {
                            name,
                            kind,
                            signature: None,
                            doc: None,
                            line: node.start_position().row + 1,
                            is_public: true,
                        });
                    }
                }
            }
        }
    }

    /// Extract JSDoc comment
    fn extract_jsdoc(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        let start_row = node.start_position().row;
        if start_row == 0 {
            return None;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut doc_lines = Vec::new();
        let mut in_jsdoc = false;

        let mut row = start_row.saturating_sub(1);
        while row < lines.len() {
            let line = lines[row].trim();

            if line.ends_with("*/") {
                in_jsdoc = true;
            } else if line.starts_with("/**") {
                if in_jsdoc {
                    doc_lines.insert(
                        0,
                        line.trim_start_matches("/**").trim_end_matches("*/").trim(),
                    );
                }
                break;
            } else if in_jsdoc {
                doc_lines.insert(0, line.trim_start_matches('*').trim());
            } else if !line.is_empty() {
                break;
            }

            if row == 0 {
                break;
            }
            row -= 1;
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// Extract JS import
    fn extract_js_import(&self, node: &tree_sitter::Node, source: &str) -> Option<Import> {
        let text = self.node_text(node, source);

        // Parse import statement (simplified)
        // Examples: import { a, b } from 'module'
        //           import default from 'module'
        //           import * as name from 'module'

        let path = text
            .split(" from ")
            .last()?
            .trim()
            .trim_matches(|c| c == '\'' || c == '"' || c == ';')
            .to_string();

        // Determine if external (doesn't start with . or /)
        let is_external = !path.starts_with('.') && !path.starts_with('/');

        Some(Import {
            path,
            items: vec![],
            is_external,
        })
    }

    /// Extract symbols and imports from Go AST nodes
    fn extract_go_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        let kind = node.kind();

        match kind {
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                    let signature = self.extract_go_function_signature(node, source);
                    let doc = self.extract_go_doc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "method_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                    let signature = self.extract_go_function_signature(node, source);
                    let doc = self.extract_go_doc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
            "type_declaration" => {
                self.extract_go_type_declaration(node, source, symbols);
            }
            "import_declaration" => {
                let import = self.extract_go_import(node, source);
                for imp in import {
                    imports.push(imp);
                }
            }
            "const_declaration" | "var_declaration" => {
                self.extract_go_const_var(node, source, symbols);
            }
            _ => {}
        }
    }

    /// Extract Go function signature
    fn extract_go_function_signature(&self, node: &tree_sitter::Node, source: &str) -> String {
        let mut signature = String::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        if let Some(result_node) = node.child_by_field_name("result") {
            signature.push_str(" ");
            signature.push_str(&self.node_text(&result_node, source));
        }

        signature
    }

    /// Extract Go doc comment
    fn extract_go_doc(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        let start_row = node.start_position().row;
        if start_row == 0 {
            return None;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut doc_lines = Vec::new();

        let mut row = start_row.saturating_sub(1);
        while row < lines.len() {
            let line = lines[row].trim();

            if line.starts_with("//") {
                doc_lines.insert(0, line.trim_start_matches("//").trim());
            } else if !line.is_empty() {
                break;
            }

            if row == 0 {
                break;
            }
            row -= 1;
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// Extract Go type declaration
    fn extract_go_type_declaration(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

                    // Determine kind based on type
                    let kind = if let Some(type_node) = child.child_by_field_name("type") {
                        match type_node.kind() {
                            "struct_type" => SymbolKind::Struct,
                            "interface_type" => SymbolKind::Interface,
                            _ => SymbolKind::Type,
                        }
                    } else {
                        SymbolKind::Type
                    };

                    symbols.push(Symbol {
                        name,
                        kind,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
        }
    }

    /// Extract Go const/var declaration
    fn extract_go_const_var(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "const_spec" || child.kind() == "var_spec" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Constant,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                    });
                }
            }
        }
    }

    /// Extract Go import
    fn extract_go_import(&self, node: &tree_sitter::Node, source: &str) -> Vec<Import> {
        let mut imports = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "import_spec" {
                let text = self.node_text(&child, source);
                let path = text.trim().trim_matches('"').to_string();

                // Determine if external (doesn't contain the current module path)
                // Simplified: assume external if it contains a dot
                let is_external = path.contains('.');

                imports.push(Import {
                    path,
                    items: vec![],
                    is_external,
                });
            } else if child.kind() == "import_spec_list" {
                let mut inner_cursor = child.walk();
                for spec in child.children(&mut inner_cursor) {
                    if spec.kind() == "import_spec" {
                        let text = self.node_text(&spec, source);
                        let path = text.trim().trim_matches('"').to_string();
                        let is_external = path.contains('.');

                        imports.push(Import {
                            path,
                            items: vec![],
                            is_external,
                        });
                    }
                }
            }
        }

        imports
    }

    /// Get text content of a node
    fn node_text(&self, node: &tree_sitter::Node, source: &str) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        source[start..end].to_string()
    }
}

impl Default for AstAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize AST analyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(
            SupportedLanguage::from_extension("rs"),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(
            SupportedLanguage::from_extension("py"),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(
            SupportedLanguage::from_extension("js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("ts"),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("go"),
            Some(SupportedLanguage::Go)
        );
        assert_eq!(SupportedLanguage::from_extension("txt"), None);
    }

    #[test]
    fn test_parse_rust_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
use std::path::Path;
use crate::models::Foo;

/// A test function
pub fn test_function(x: i32) -> String {
    x.to_string()
}

struct TestStruct {
    field: String,
}

pub enum TestEnum {
    A,
    B,
}
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.rs"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.name, "test");
        assert_eq!(module.language, SupportedLanguage::Rust);

        // Check symbols
        let function = module
            .symbols
            .iter()
            .find(|s| s.name == "test_function")
            .expect("Should find test_function");
        assert_eq!(function.kind, SymbolKind::Function);
        assert!(function.is_public);
        assert!(function.signature.as_ref().unwrap().contains("i32"));

        let struct_symbol = module
            .symbols
            .iter()
            .find(|s| s.name == "TestStruct")
            .expect("Should find TestStruct");
        assert_eq!(struct_symbol.kind, SymbolKind::Struct);

        let enum_symbol = module
            .symbols
            .iter()
            .find(|s| s.name == "TestEnum")
            .expect("Should find TestEnum");
        assert_eq!(enum_symbol.kind, SymbolKind::Enum);

        // Check imports
        assert!(module.imports.len() >= 2);
        let external_import = module.imports.iter().find(|i| i.path.contains("std"));
        assert!(external_import.is_some());
    }

    #[test]
    fn test_parse_python_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
import os
from typing import List

def public_function(x: int) -> str:
    """This is a docstring."""
    return str(x)

def _private_function():
    pass

class TestClass:
    """A test class."""
    def method(self):
        pass
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.py"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.language, SupportedLanguage::Python);

        let public_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "public_function")
            .expect("Should find public_function");
        assert!(public_fn.is_public);
        assert!(public_fn.doc.is_some());

        let private_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "_private_function")
            .expect("Should find _private_function");
        assert!(!private_fn.is_public);

        let class = module
            .symbols
            .iter()
            .find(|s| s.name == "TestClass")
            .expect("Should find TestClass");
        assert_eq!(class.kind, SymbolKind::Class);
    }

    #[test]
    fn test_parse_javascript_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
import { something } from './module';
import external from 'external-package';

function regularFunction(x) {
    return x;
}

const arrowFunction = (y) => y * 2;

class TestClass {
    constructor() {}
}
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.js"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.language, SupportedLanguage::JavaScript);

        // Check function was found
        assert!(module
            .symbols
            .iter()
            .any(|s| s.name == "regularFunction"));
        assert!(module.symbols.iter().any(|s| s.name == "TestClass"));

        // Check imports
        let internal_import = module.imports.iter().find(|i| i.path == "./module");
        assert!(internal_import.is_some());
        assert!(!internal_import.unwrap().is_external);

        let external_import = module
            .imports
            .iter()
            .find(|i| i.path == "external-package");
        assert!(external_import.is_some());
        assert!(external_import.unwrap().is_external);
    }

    #[test]
    fn test_parse_go_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
package main

import (
    "fmt"
    "github.com/example/pkg"
)

// PublicFunction is exported
func PublicFunction(x int) string {
    return fmt.Sprintf("%d", x)
}

func privateFunction() {
}

type PublicStruct struct {
    Field string
}

type PublicInterface interface {
    Method()
}
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.go"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.language, SupportedLanguage::Go);

        let public_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "PublicFunction")
            .expect("Should find PublicFunction");
        assert!(public_fn.is_public);

        let private_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "privateFunction")
            .expect("Should find privateFunction");
        assert!(!private_fn.is_public);

        let public_struct = module
            .symbols
            .iter()
            .find(|s| s.name == "PublicStruct")
            .expect("Should find PublicStruct");
        assert_eq!(public_struct.kind, SymbolKind::Struct);
        assert!(public_struct.is_public);

        let public_interface = module
            .symbols
            .iter()
            .find(|s| s.name == "PublicInterface")
            .expect("Should find PublicInterface");
        assert_eq!(public_interface.kind, SymbolKind::Interface);
    }

    #[test]
    fn test_unsupported_extension() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let result = analyzer.parse_file(&PathBuf::from("test.txt"), "some text");
        assert!(result.is_err());
    }

    #[test]
    fn test_analysis_context() {
        let mut context = AnalysisContext::new();

        context.modules.push(ModuleInfo {
            name: "test".to_string(),
            path: "test.rs".to_string(),
            language: SupportedLanguage::Rust,
            symbols: vec![
                Symbol {
                    name: "fn1".to_string(),
                    kind: SymbolKind::Function,
                    signature: None,
                    doc: None,
                    line: 1,
                    is_public: true,
                },
                Symbol {
                    name: "fn2".to_string(),
                    kind: SymbolKind::Function,
                    signature: None,
                    doc: None,
                    line: 5,
                    is_public: true,
                },
            ],
            imports: vec![
                Import {
                    path: "std::path".to_string(),
                    items: vec![],
                    is_external: true,
                },
                Import {
                    path: "crate::models".to_string(),
                    items: vec![],
                    is_external: false,
                },
            ],
        });

        assert_eq!(context.total_symbols(), 2);
        assert_eq!(context.external_dependencies(), vec!["std::path"]);
    }
}
