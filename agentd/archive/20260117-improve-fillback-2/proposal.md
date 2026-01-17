# Change: improve-fillback-2

## Summary

Improve the `fillback` command by adding local pre-processing with `tree-sitter` for AST-based code analysis, introducing an interactive clarification phase, and generating language-agnostic specifications directly into the `agentd/specs/` directory.

## Why

- **Deeper Analysis**: Raw text analysis by LLMs can miss structural nuances. Local AST parsing provides a precise view of modules, functions, and their relationships.
- **Language Agnostic**: Standardized formats like Mermaid, JSON Schema, and OpenAPI make specifications more portable and easier to validate.
- **Improved Workflow**: Writing directly to the main specs directory (with user confirmation) simplifies the process of documenting an existing codebase.
- **Accuracy**: A clarification phase ensures the generated specs align with the developer's intent and architectural understanding.

## What Changes

- **AST Parsing Layer**: Integrate `tree-sitter` to parse multiple languages (Rust, Python, TS/JS, Go, etc.).
- **Dependency Analysis**: Implement logic to build a dependency graph and identify module boundaries using AST data.
- **Interactive CLI**: Update `fillback` command to include a clarification step where the AI asks questions about the codebase before generation.
- **Targeted Output**: Change default output location to `agentd/specs/` instead of `agentd/changes/<id>/specs/`.
- **Incremental Updates**: Add logic to detect existing specs and prompt the user for confirmation before updating or overwriting.
- **Enhanced Prompting**: Update orchestrator prompts to favor Mermaid, JSON Schema, Pseudo code, OpenAPI, and AsyncAPI.

## Impact

- Affected specs: `agentd/specs/workflows.md` (to reflect the new fillback flow)
- Affected code:
  - `src/fillback/code.rs`: Core logic for code analysis and spec generation.
  - `src/fillback/mod.rs`: Module exports and new components (ast, graph).
  - `src/fillback/ast.rs`: New AST analysis module using tree-sitter.
  - `src/fillback/graph.rs`: New dependency graph extraction module.
  - `src/cli/fillback.rs`: CLI command flow and interactivity.
  - `src/main.rs`: Update CLI argument structure (remove change_id).
  - `Cargo.toml`: Addition of `tree-sitter` and language-specific grammar crates.
  - `templates/gemini/commands/agentd/fillback.toml`: Update prompt for language-agnostic output.
- Breaking changes: **CLI interface change** - `change_id` parameter removed.