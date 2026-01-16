# Change: test-retry

## Summary

Add a dedicated `agentd archived` CLI command to list completed and archived changes with enhanced details like date and summary.

## Why

Currently, listing archived changes is hidden behind a flag (`agentd list --archived`). Users often want to browse their project history as a first-class action. A dedicated `archived` command improves discoverability and provides a platform for richer historical data display compared to the simple directory listing provided by the current `list` flag.

## What Changes

- **CLI Layer**:
    - Add `Archived` as a top-level subcommand in `src/main.rs`.
    - Update `List` subcommand help to mention the new `archived` command.
- **Logic Layer**:
    - Enhance `src/cli/list.rs` to support a detailed view for archived changes (default for the `archived` command).
    - Implement a simple parser in `src/parser/markdown.rs` to extract the "Summary" section from archived `proposal.md` files.
    - Format the output to show: Date, ID, and Summary.

## Impact

- Affected specs: `specs/archived-command.md`
- Affected code: `src/main.rs`, `src/cli/list.rs`, `src/parser/markdown.rs`
- Breaking changes: No
