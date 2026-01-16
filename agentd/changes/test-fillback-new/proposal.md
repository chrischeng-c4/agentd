# Change: test-fillback-new

## Summary

Add functionality to list active and archived changes, including a detailed view for archived changes with metadata extraction.

## Why

Users need a way to track their current progress and review past work. As the number of changes grows, a simple directory listing is insufficient. Providing a detailed view that extracts summaries from proposals helps users quickly identify completed work without opening individual files.

## What Changes

- Add `list` command to show active changes by default.
- Add `--archived` flag to `list` command for a brief list of archived changes.
- Add `archived` command for a detailed tabular view of archived changes.
- Implement metadata extraction from `proposal.md` (specifically the "Summary" section).
- Implement date parsing and sorting for archived changes.

## Impact

- Affected specs: `specs/list-changes.md`
- Affected code: `src/cli/list.rs`, `src/cli/mod.rs`, `src/main.rs`, `src/parser/markdown.rs`
- Breaking changes: No
