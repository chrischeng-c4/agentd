# Change: fillback

## Summary
Introduces the `fillback` command to reverse-engineer Agentd specifications from existing codebases and migrate specifications from other formats (OpenSpec, GitHub Speckit).

## Why
Adopting Agentd in an existing project or migrating from other spec-driven tools is currently manual and high-friction. Users need a way to "bootstrap" their Agentd project by importing existing knowledge, whether it exists as structured specs in other formats or implicitly within the codebase itself. This lowers the barrier to entry and enables immediate value from Agentd features like validation and change management.

## What Changes
- **New CLI Command:** `agentd fillback` with options to specify source path and import strategy.
- **Import Strategies:**
  - `openspec`: Converts OpenSpec YAML/JSON files to Agentd Markdown specs.
  - `speckit`: Converts GitHub Speckit Markdown files to Agentd specs.
  - `code`: Uses LLM orchestration to analyze source code and generate high-level technical designs and specs.
- **Output:** Generates files in `agentd/changes/<change_id>/` (standard Agentd change structure).

## Impact
- **Affected Specs:** None (New feature).
- **Affected Code:**
  - `src/cli/mod.rs`: Registration of new command.
  - `src/cli/fillback.rs`: New command implementation.
  - `src/fillback/`: New module (implied) for migration logic, or expansion of `src/parser/`.
  - `Cargo.toml`: Potential new dependencies for specific parsers if needed (though existing markdown/yaml parsers might suffice).
- **Breaking Changes:** No.