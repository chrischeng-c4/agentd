# Implementation Notes: improve-fillback-2

## Summary

Enhanced the `fillback` command with tree-sitter AST parsing, interactive clarification, and language-agnostic specification generation.

## Changes Made

### 1. Dependencies (Cargo.toml)

Added tree-sitter and language grammar crates:
- `tree-sitter = "0.24"`
- `tree-sitter-rust = "0.23"`
- `tree-sitter-python = "0.23"`
- `tree-sitter-javascript = "0.23"`
- `tree-sitter-typescript = "0.23"`
- `tree-sitter-go = "0.23"`

### 2. New Modules

#### `src/fillback/ast.rs`
- `AstAnalyzer`: Tree-sitter based parser for multiple languages
- `SupportedLanguage`: Enum for Rust, Python, JavaScript, TypeScript, Go
- `Symbol`: Extracted function/struct/enum/class with metadata
- `Import`: Dependency relationship (internal vs external)
- `ModuleInfo`: Parsed module with symbols and imports
- `AnalysisContext`: Aggregated analysis results

Capabilities:
- Extracts functions, structs, enums, classes, interfaces, types, constants
- Parses doc comments (Rust ///, Python docstrings, JSDoc, Go //)
- Detects visibility (pub in Rust, _ prefix in Python, capitalization in Go)
- Identifies external vs internal imports

#### `src/fillback/graph.rs`
- `DependencyGraph`: Module dependency graph with nodes and edges
- `ModuleNode`: Represents a module with symbol counts
- `Dependency`: Edge with type (import, call, inheritance)
- `GraphStats`: Statistics about the graph

Outputs:
- Mermaid flowchart diagrams
- Markdown documentation
- Console-friendly compact view

### 3. Updated Modules

#### `src/fillback/code.rs`
Complete rewrite with:
- `CodeStrategyConfig`: Configuration for path, module filter, force mode
- `analyze_codebase()`: AST-based analysis
- `display_summary()`: Shows analysis results
- `display_dependency_graph()`: Mermaid output to console
- `run_clarification()`: Interactive questions using dialoguer
- `check_existing_specs()`: Detects existing specs
- `confirm_overwrite()`: User confirmation for overwrites
- `generate_specs()`: Creates markdown spec files

#### `src/fillback/mod.rs`
Added exports for new modules and types.

#### `src/main.rs`
Changed CLI interface:
- Removed: `change_id` positional argument
- Added: `--path` (source directory)
- Added: `--module` (filter specific module)
- Added: `--force` (overwrite without confirmation)

#### `src/cli/fillback.rs`
Updated to use new CodeStrategy with configuration.
Output now goes to `agentd/specs/` instead of `agentd/changes/<id>/`.

#### `templates/gemini/commands/agentd/fillback.toml`
Updated prompt to emphasize language-agnostic output:
- Mermaid diagrams
- JSON Schema
- Pseudo code
- OpenAPI/AsyncAPI
- WHEN/THEN acceptance criteria

### 4. Tests

#### Unit Tests (in-module)
- `ast.rs`: 7 tests for parsing each language and edge cases
- `graph.rs`: 8 tests for graph construction and output
- `code.rs`: 10 tests for analysis and spec generation

#### Integration Tests (`tests/fillback_test.rs`)
14 tests covering:
- Multi-language project analysis
- Dependency graph generation
- Mermaid output format
- Spec file generation
- Module filtering
- Parse error handling
- Force overwrite mode
- Individual language AST parsing

## Test Results

```
running 200 tests (library)
test result: ok. 200 passed; 0 failed

running 14 tests (integration)
test result: ok. 14 passed; 0 failed
```

## Breaking Changes

- CLI interface changed: `agentd fillback <change-id>` → `agentd fillback [--path] [--module] [--force]`
- Output location changed: `agentd/changes/<id>/specs/` → `agentd/specs/`
- No longer creates a change directory

## Usage

```bash
# Analyze current directory
agentd fillback

# Analyze specific path
agentd fillback --path ./src

# Filter to specific module
agentd fillback --module auth

# Force overwrite existing specs
agentd fillback --force
```

## Generated Output

The command produces:
1. `agentd/specs/_dependency-graph.md` - Module relationship diagram
2. `agentd/specs/_overview.md` - Project overview with architecture
3. `agentd/specs/<module>.md` - Per-module specifications

Each module spec includes:
- Symbol table (functions, types, visibility)
- Interface definitions (pseudo code)
- Dependencies (internal and external)
