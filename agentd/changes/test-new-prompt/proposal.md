# Change: test-new-prompt

## Summary
Implement a new `search` command for the Agentd CLI to allow users to quickly find information across specifications and active changes. This serves as a test case for the improved proposal prompt.

## Why
As the number of specifications and changes grows, it becomes difficult for users and agents to locate specific requirements, data models, or architectural decisions. A dedicated `search` command will improve discoverability and reduce duplication by allowing keyword and regex searches across the `agentd/` directory.

## What Changes
- **CLI Subcommand**: Add `search` to the `agentd` CLI.
- **Search Logic**: Implement a search engine that indexes (or scans) `.md` files in `agentd/specs/` and `agentd/changes/`.
- **Output Formatting**: Display search results with context (line numbers, surrounding text) and links to the files.
- **Filtering**: Support filtering by change ID, spec type, or date.

## Impact
- Affected specs: `cli-commands.md` (new), `search-engine.md` (new)
- Affected code: `src/cli/search.rs`, `src/cli/mod.rs`, `src/main.rs`
- Breaking changes: No