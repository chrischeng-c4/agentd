# Changelog

## [Unreleased]

### Added
- **Multi-language AST Parsing**: `fillback` now uses `tree-sitter` to provide deep analysis of code in Rust, Python, JavaScript, TypeScript, and Go.
- **Dependency Graph**: Automatically generates Mermaid flowcharts showing module dependencies and structural relationships.
- **Interactive Clarification**: The CLI now asks clarifying questions based on AST analysis to ensure generated specs match developer intent.
- **Incremental Updates**: Detects existing specifications in `agentd/specs/` and prompts for confirmation before overwriting, preventing data loss.
- **Module Filtering**: Added `--module` flag to target analysis and generation to specific modules.
- **Force Mode**: Added `--force` flag to bypass confirmation prompts.

### Changed
- **CLI Interface**: `agentd fillback` command signature has changed. The `change_id` argument is removed.
  - Old: `agentd fillback <change_id>`
  - New: `agentd fillback [--path <path>] [--module <name>]`
- **Output Location**: Specifications are now generated directly into the project's `specs/` directory (or configured path) instead of a temporary change directory.
- **Spec Format**: Updated generation templates to favor language-agnostic formats (Mermaid, JSON Schema, Pseudo-code) over raw text.

### Fixed
- Improved error handling for parsing failures; the tool now logs warnings and continues instead of crashing on malformed files.
