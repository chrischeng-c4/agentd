# Tasks

## 1. Data & Infrastructure Layer

- [x] 1.1 Add dependencies to Cargo.toml
  - File: `Cargo.toml` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r5-multi-language-support`
  - Do: Add `tree-sitter`, `tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-javascript`, `tree-sitter-typescript`, `tree-sitter-go`.
  - Depends: none

- [x] 1.2 Implement AST Analysis Module
  - File: `src/fillback/ast.rs` (CREATE)
  - Spec: `specs/fillback-enhancement.md#r1-ast-based-local-analysis`
  - Do: Implement tree-sitter based parsing for supported languages. Extract modules, functions, structs, and imports.
  - Depends: 1.1

- [x] 1.3 Implement Dependency Graph Extractor
  - File: `src/fillback/graph.rs` (CREATE)
  - Spec: `specs/fillback-enhancement.md#r7-dependency-graph-output`
  - Do: Build module dependency graph from AST data. Output as Mermaid flowchart.
  - Depends: 1.2

- [x] 1.4 Update Module Exports
  - File: `src/fillback/mod.rs` (MODIFY)
  - Do: Export new modules `ast` and `graph`.
  - Depends: 1.2, 1.3

## 2. Logic Layer

- [x] 2.1 Implement Analysis Summary
  - File: `src/fillback/code.rs` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r2-interactive-clarification`
  - Do: Generate and display initial understanding summary before asking clarification questions.
  - Depends: 1.4

- [x] 2.2 Implement Clarification Phase
  - File: `src/fillback/code.rs` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r2-interactive-clarification`
  - Do: Call LLM for clarification questions based on AST analysis. Wait for user input.
  - Depends: 2.1

- [x] 2.3 Update Fillback Prompt Template
  - File: `templates/gemini/commands/agentd/fillback.toml` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r3-language-agnostic-output`
  - Do: Update prompt to enforce Mermaid, JSON Schema, Pseudo code, OpenAPI/AsyncAPI output formats.
  - Depends: none

- [x] 2.4 Implement Incremental Update Logic
  - File: `src/fillback/code.rs` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r4-direct-and-incremental-updates`
  - Do: Check for existing files in `agentd/specs/`, show diff summary, prompt for confirmation.
  - Depends: none

- [x] 2.5 Implement Error Handling
  - File: `src/fillback/code.rs` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r8-error-handling`
  - Do: Handle tree-sitter parse failures gracefully. Log warnings and continue with other files.
  - Depends: 1.2

## 3. Integration Layer

- [x] 3.1 Update CLI Command Arguments
  - File: `src/main.rs` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r6-cli-interface-change`
  - Do: Remove `change_id` parameter. Add `--path`, `--module`, `--force` flags.
  - Depends: none

- [x] 3.2 Update CLI Command Logic
  - File: `src/cli/fillback.rs` (MODIFY)
  - Spec: `specs/fillback-enhancement.md#r6-cli-interface-change`
  - Do: Change default output to `agentd/specs/`. Remove change directory creation.
  - Depends: 3.1, 2.4

- [x] 3.3 Wire up Strategy Factory
  - File: `src/fillback/factory.rs` (MODIFY)
  - Do: Initialize `CodeStrategy` with new AST and graph components.
  - Depends: 3.2

## 4. Testing

- [x] 4.1 Unit Tests for AST Analysis
  - File: `src/fillback/ast.rs` (MODIFY)
  - Verify: `specs/fillback-enhancement.md#scenario-initial-fillback-of-a-rust-project`
  - Depends: 1.2

- [x] 4.2 Unit Tests for Dependency Graph
  - File: `src/fillback/graph.rs` (MODIFY)
  - Verify: `specs/fillback-enhancement.md#scenario-dependency-graph-generation`
  - Depends: 1.3

- [x] 4.3 Integration Test for Fillback Flow
  - File: `tests/fillback_test.rs` (CREATE)
  - Verify: `specs/fillback-enhancement.md#acceptance-criteria`
  - Depends: 3.3

- [x] 4.4 Test Error Handling
  - File: `tests/fillback_test.rs` (MODIFY)
  - Verify: `specs/fillback-enhancement.md#scenario-parse-error-handling`
  - Depends: 4.3
