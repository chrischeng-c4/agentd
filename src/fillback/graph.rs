//! Dependency Graph Module
//!
//! Builds module dependency graphs from AST analysis data.
//! Outputs graphs as Mermaid flowcharts for visualization.

use crate::fillback::ast::AnalysisContext;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Type of dependency relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Import,
    Call,
    Inheritance,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import => write!(f, "import"),
            Self::Call => write!(f, "call"),
            Self::Inheritance => write!(f, "inheritance"),
        }
    }
}

/// A dependency edge between modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from: String,
    pub to: String,
    pub dependency_type: DependencyType,
}

/// Module node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNode {
    pub name: String,
    pub path: String,
    pub is_external: bool,
    pub symbol_count: usize,
    pub public_symbol_count: usize,
}

/// Complete dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<ModuleNode>,
    pub edges: Vec<Dependency>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Build dependency graph from analysis context
    pub fn from_analysis(context: &AnalysisContext) -> Self {
        let mut graph = Self::new();
        let mut node_names: HashSet<String> = HashSet::new();
        let mut external_deps: HashSet<String> = HashSet::new();

        // Add all analyzed modules as nodes
        for module in &context.modules {
            let public_count = module.symbols.iter().filter(|s| s.is_public).count();

            graph.nodes.push(ModuleNode {
                name: module.name.clone(),
                path: module.path.clone(),
                is_external: false,
                symbol_count: module.symbols.len(),
                public_symbol_count: public_count,
            });
            node_names.insert(module.name.clone());

            // Collect external dependencies
            for import in &module.imports {
                if import.is_external {
                    external_deps.insert(Self::extract_base_module(&import.path));
                }
            }
        }

        // Add external dependency nodes
        for dep in external_deps {
            if !node_names.contains(&dep) {
                graph.nodes.push(ModuleNode {
                    name: dep.clone(),
                    path: dep.clone(),
                    is_external: true,
                    symbol_count: 0,
                    public_symbol_count: 0,
                });
                node_names.insert(dep);
            }
        }

        // Build edges from imports
        for module in &context.modules {
            for import in &module.imports {
                let target = Self::extract_base_module(&import.path);

                graph.edges.push(Dependency {
                    from: module.name.clone(),
                    to: target,
                    dependency_type: DependencyType::Import,
                });
            }
        }

        // Deduplicate edges
        graph.deduplicate_edges();

        graph
    }

    /// Extract base module name from import path
    fn extract_base_module(path: &str) -> String {
        // Handle different import formats:
        // - Rust: std::path::Path -> std
        // - Python: os.path -> os
        // - JS: ./module -> module
        // - Go: github.com/user/pkg -> github.com/user/pkg

        let path = path.trim();

        // Handle relative imports
        if path.starts_with('.') {
            return path
                .trim_start_matches("./")
                .trim_start_matches("../")
                .split('/')
                .next()
                .unwrap_or(path)
                .to_string();
        }

        // Handle Rust crate:: imports
        if path.starts_with("crate::") {
            return path
                .trim_start_matches("crate::")
                .split("::")
                .next()
                .unwrap_or(path)
                .to_string();
        }

        // Handle Rust self:: and super:: imports
        if path.starts_with("self::") || path.starts_with("super::") {
            return path.split("::").next().unwrap_or(path).to_string();
        }

        // For Rust std library and external crates
        if path.contains("::") {
            return path.split("::").next().unwrap_or(path).to_string();
        }

        // For Python/JS dot notation
        if path.contains('.') && !path.contains('/') {
            return path.split('.').next().unwrap_or(path).to_string();
        }

        // For Go-style paths with slashes
        if path.contains('/') {
            // Return the full path for Go imports (they're meaningful as a whole)
            return path.to_string();
        }

        path.to_string()
    }

    /// Deduplicate edges (keep unique from-to-type combinations)
    fn deduplicate_edges(&mut self) {
        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        self.edges.retain(|edge| {
            let key = (
                edge.from.clone(),
                edge.to.clone(),
                edge.dependency_type.to_string(),
            );
            seen.insert(key)
        });
    }

    /// Get all internal modules (non-external)
    pub fn internal_modules(&self) -> Vec<&ModuleNode> {
        self.nodes.iter().filter(|n| !n.is_external).collect()
    }

    /// Get all external dependencies
    pub fn external_dependencies(&self) -> Vec<&ModuleNode> {
        self.nodes.iter().filter(|n| n.is_external).collect()
    }

    /// Generate Mermaid flowchart diagram
    pub fn to_mermaid(&self) -> String {
        let mut output = String::new();
        output.push_str("```mermaid\nflowchart TD\n");

        // Add subgraph for internal modules
        let internal = self.internal_modules();
        if !internal.is_empty() {
            output.push_str("    subgraph Internal[\"Internal Modules\"]\n");
            for node in &internal {
                let id = Self::sanitize_id(&node.name);
                let label = if node.symbol_count > 0 {
                    format!(
                        "{}\\n({} symbols, {} public)",
                        node.name, node.symbol_count, node.public_symbol_count
                    )
                } else {
                    node.name.clone()
                };
                output.push_str(&format!("        {}[\"{}\"]\n", id, label));
            }
            output.push_str("    end\n");
        }

        // Add subgraph for external dependencies
        let external = self.external_dependencies();
        if !external.is_empty() {
            output.push_str("    subgraph External[\"External Dependencies\"]\n");
            for node in &external {
                let id = Self::sanitize_id(&node.name);
                output.push_str(&format!("        {}[/\"{}\"/]\n", id, node.name));
            }
            output.push_str("    end\n");
        }

        // Add edges
        for edge in &self.edges {
            let from_id = Self::sanitize_id(&edge.from);
            let to_id = Self::sanitize_id(&edge.to);

            let arrow = match edge.dependency_type {
                DependencyType::Import => "-->",
                DependencyType::Call => "-.->",
                DependencyType::Inheritance => "===>",
            };

            output.push_str(&format!("    {} {} {}\n", from_id, arrow, to_id));
        }

        output.push_str("```\n");
        output
    }

    /// Generate a compact Mermaid diagram for console output
    pub fn to_mermaid_compact(&self) -> String {
        let mut output = String::new();
        output.push_str("flowchart LR\n");

        // Only show internal modules and their direct dependencies
        for node in self.internal_modules() {
            let id = Self::sanitize_id(&node.name);
            output.push_str(&format!("    {}[{}]\n", id, node.name));
        }

        // Show unique external dependencies (deduplicated)
        let mut shown_external: HashSet<String> = HashSet::new();
        for edge in &self.edges {
            if self.nodes.iter().any(|n| n.name == edge.to && n.is_external) {
                if shown_external.insert(edge.to.clone()) {
                    let id = Self::sanitize_id(&edge.to);
                    output.push_str(&format!("    {}[/{}\\]\n", id, edge.to));
                }
            }
        }

        // Add edges (limit to avoid clutter)
        let edge_limit = 50;
        for (i, edge) in self.edges.iter().enumerate() {
            if i >= edge_limit {
                output.push_str(&format!(
                    "    %% ... and {} more edges\n",
                    self.edges.len() - edge_limit
                ));
                break;
            }
            let from_id = Self::sanitize_id(&edge.from);
            let to_id = Self::sanitize_id(&edge.to);
            output.push_str(&format!("    {} --> {}\n", from_id, to_id));
        }

        output
    }

    /// Generate markdown file content with dependency graph
    pub fn to_markdown(&self, project_name: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Dependency Graph: {}\n\n", project_name));

        output.push_str("## Overview\n\n");
        output.push_str(&format!(
            "- **Internal Modules**: {}\n",
            self.internal_modules().len()
        ));
        output.push_str(&format!(
            "- **External Dependencies**: {}\n",
            self.external_dependencies().len()
        ));
        output.push_str(&format!("- **Total Relationships**: {}\n\n", self.edges.len()));

        output.push_str("## Module Graph\n\n");
        output.push_str(&self.to_mermaid());

        output.push_str("\n## Internal Modules\n\n");
        output.push_str("| Module | Path | Symbols | Public |\n");
        output.push_str("|--------|------|---------|--------|\n");
        for node in self.internal_modules() {
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                node.name, node.path, node.symbol_count, node.public_symbol_count
            ));
        }

        output.push_str("\n## External Dependencies\n\n");
        if self.external_dependencies().is_empty() {
            output.push_str("No external dependencies detected.\n");
        } else {
            output.push_str("| Dependency |\n");
            output.push_str("|------------|\n");
            for node in self.external_dependencies() {
                output.push_str(&format!("| {} |\n", node.name));
            }
        }

        output.push_str("\n## Dependency Details\n\n");
        output.push_str("| From | To | Type |\n");
        output.push_str("|------|-----|------|\n");
        for edge in &self.edges {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                edge.from, edge.to, edge.dependency_type
            ));
        }

        output
    }

    /// Sanitize string for use as Mermaid node ID
    fn sanitize_id(name: &str) -> String {
        name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_modules: usize,
    pub internal_modules: usize,
    pub external_dependencies: usize,
    pub total_edges: usize,
    pub avg_dependencies_per_module: f64,
    pub most_connected_modules: Vec<(String, usize)>,
}

impl GraphStats {
    /// Calculate statistics from a dependency graph
    pub fn from_graph(graph: &DependencyGraph) -> Self {
        let internal_count = graph.internal_modules().len();
        let edge_count = graph.edges.len();

        // Count outgoing edges per module
        let mut outgoing_count: HashMap<String, usize> = HashMap::new();
        for edge in &graph.edges {
            *outgoing_count.entry(edge.from.clone()).or_insert(0) += 1;
        }

        // Get most connected modules
        let mut most_connected: Vec<(String, usize)> = outgoing_count.into_iter().collect();
        most_connected.sort_by(|a, b| b.1.cmp(&a.1));
        most_connected.truncate(5);

        let avg_deps = if internal_count > 0 {
            edge_count as f64 / internal_count as f64
        } else {
            0.0
        };

        Self {
            total_modules: graph.nodes.len(),
            internal_modules: internal_count,
            external_dependencies: graph.external_dependencies().len(),
            total_edges: edge_count,
            avg_dependencies_per_module: avg_deps,
            most_connected_modules: most_connected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fillback::ast::{Import, ModuleInfo, SupportedLanguage, Symbol, SymbolKind};

    fn create_test_context() -> AnalysisContext {
        let mut context = AnalysisContext::new();

        // Module A - depends on B and external
        context.modules.push(ModuleInfo {
            name: "module_a".to_string(),
            path: "src/module_a.rs".to_string(),
            language: SupportedLanguage::Rust,
            symbols: vec![
                Symbol {
                    name: "func_a".to_string(),
                    kind: SymbolKind::Function,
                    signature: Some("func_a() -> ()".to_string()),
                    doc: None,
                    line: 1,
                    is_public: true,
                },
                Symbol {
                    name: "private_func".to_string(),
                    kind: SymbolKind::Function,
                    signature: None,
                    doc: None,
                    line: 5,
                    is_public: false,
                },
            ],
            imports: vec![
                Import {
                    path: "crate::module_b".to_string(),
                    items: vec![],
                    is_external: false,
                },
                Import {
                    path: "std::collections::HashMap".to_string(),
                    items: vec![],
                    is_external: true,
                },
            ],
        });

        // Module B - depends on external only
        context.modules.push(ModuleInfo {
            name: "module_b".to_string(),
            path: "src/module_b.rs".to_string(),
            language: SupportedLanguage::Rust,
            symbols: vec![Symbol {
                name: "func_b".to_string(),
                kind: SymbolKind::Function,
                signature: None,
                doc: None,
                line: 1,
                is_public: true,
            }],
            imports: vec![Import {
                path: "serde::Serialize".to_string(),
                items: vec![],
                is_external: true,
            }],
        });

        // Module C - depends on A and B
        context.modules.push(ModuleInfo {
            name: "module_c".to_string(),
            path: "src/module_c.rs".to_string(),
            language: SupportedLanguage::Rust,
            symbols: vec![],
            imports: vec![
                Import {
                    path: "crate::module_a".to_string(),
                    items: vec![],
                    is_external: false,
                },
                Import {
                    path: "crate::module_b".to_string(),
                    items: vec![],
                    is_external: false,
                },
            ],
        });

        context
    }

    #[test]
    fn test_build_dependency_graph() {
        let context = create_test_context();
        let graph = DependencyGraph::from_analysis(&context);

        // Should have 5 nodes: 3 internal + 2 external (std, serde)
        assert_eq!(graph.nodes.len(), 5);
        assert_eq!(graph.internal_modules().len(), 3);
        assert_eq!(graph.external_dependencies().len(), 2);

        // Check edges
        assert!(!graph.edges.is_empty());

        // module_a should depend on module_b and std
        let module_a_deps: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.from == "module_a")
            .collect();
        assert_eq!(module_a_deps.len(), 2);
    }

    #[test]
    fn test_extract_base_module() {
        // Rust paths
        assert_eq!(
            DependencyGraph::extract_base_module("std::collections::HashMap"),
            "std"
        );
        assert_eq!(
            DependencyGraph::extract_base_module("crate::module_a"),
            "module_a"
        );
        assert_eq!(
            DependencyGraph::extract_base_module("self::submodule"),
            "self"
        );

        // Python paths
        assert_eq!(
            DependencyGraph::extract_base_module("os.path"),
            "os"
        );
        assert_eq!(
            DependencyGraph::extract_base_module("typing"),
            "typing"
        );

        // JS/TS paths
        assert_eq!(
            DependencyGraph::extract_base_module("./module"),
            "module"
        );
        assert_eq!(
            DependencyGraph::extract_base_module("../utils/helper"),
            "utils"
        );
        assert_eq!(
            DependencyGraph::extract_base_module("lodash"),
            "lodash"
        );

        // Go paths
        assert_eq!(
            DependencyGraph::extract_base_module("github.com/user/pkg"),
            "github.com/user/pkg"
        );
        assert_eq!(
            DependencyGraph::extract_base_module("fmt"),
            "fmt"
        );
    }

    #[test]
    fn test_to_mermaid() {
        let context = create_test_context();
        let graph = DependencyGraph::from_analysis(&context);
        let mermaid = graph.to_mermaid();

        // Should contain mermaid code block markers
        assert!(mermaid.contains("```mermaid"));
        assert!(mermaid.contains("flowchart TD"));
        assert!(mermaid.contains("```"));

        // Should contain subgraphs
        assert!(mermaid.contains("subgraph Internal"));
        assert!(mermaid.contains("subgraph External"));

        // Should contain module names
        assert!(mermaid.contains("module_a"));
        assert!(mermaid.contains("module_b"));
        assert!(mermaid.contains("module_c"));
    }

    #[test]
    fn test_to_markdown() {
        let context = create_test_context();
        let graph = DependencyGraph::from_analysis(&context);
        let markdown = graph.to_markdown("Test Project");

        // Should contain headers
        assert!(markdown.contains("# Dependency Graph: Test Project"));
        assert!(markdown.contains("## Overview"));
        assert!(markdown.contains("## Module Graph"));
        assert!(markdown.contains("## Internal Modules"));
        assert!(markdown.contains("## External Dependencies"));

        // Should contain statistics
        assert!(markdown.contains("Internal Modules"));
        assert!(markdown.contains("External Dependencies"));
    }

    #[test]
    fn test_graph_stats() {
        let context = create_test_context();
        let graph = DependencyGraph::from_analysis(&context);
        let stats = GraphStats::from_graph(&graph);

        assert_eq!(stats.internal_modules, 3);
        assert_eq!(stats.external_dependencies, 2);
        assert!(stats.total_edges > 0);
        assert!(stats.avg_dependencies_per_module > 0.0);
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(DependencyGraph::sanitize_id("module_a"), "module_a");
        assert_eq!(
            DependencyGraph::sanitize_id("github.com/user/pkg"),
            "github_com_user_pkg"
        );
        assert_eq!(
            DependencyGraph::sanitize_id("std::collections"),
            "std__collections"
        );
    }

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());

        let mermaid = graph.to_mermaid();
        assert!(mermaid.contains("```mermaid"));
    }

    #[test]
    fn test_compact_mermaid() {
        let context = create_test_context();
        let graph = DependencyGraph::from_analysis(&context);
        let compact = graph.to_mermaid_compact();

        // Should use LR (left-right) layout for compact view
        assert!(compact.contains("flowchart LR"));

        // Should contain module nodes
        assert!(compact.contains("module_a"));
    }
}
