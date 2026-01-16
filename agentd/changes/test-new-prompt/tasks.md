# Tasks

## 1. CLI Layer
- [ ] 1.1 Create search subcommand definition
  - File: `src/cli/search.rs` (CREATE)
  - Spec: `specs/search-command.md#flow`
  - Do: Define the `Search` struct for Clap with arguments for query and filters.
- [ ] 1.2 Register search command in mod.rs
  - File: `src/cli/mod.rs` (MODIFY)
  - Spec: `specs/search-command.md#flow`
  - Do: Add `pub mod search;` to the module list.
- [ ] 1.3 Add search command to main Cli enum
  - File: `src/main.rs` (MODIFY)
  - Spec: `specs/search-command.md#flow`
  - Do: Add `Search` variant to `Commands` enum and dispatch to `cli::search::handle`.

## 2. Logic Layer
- [ ] 2.1 Implement search engine core
  - File: `src/search/mod.rs` (CREATE)
  - Spec: `specs/search-command.md#interfaces`
  - Do: Implement the `search_files` function using `grep`-like logic or the `ripgrep` crate if available.
- [ ] 2.2 Implement result formatting
  - File: `src/ui/tables.rs` (MODIFY) or `src/cli/search.rs` (MODIFY)
  - Spec: `specs/search-command.md#interfaces`
  - Do: Implement `format_results` to print matches with line numbers and context.

## 3. Testing
- [ ] 3.1 Unit tests for search logic
  - File: `src/search/mod.rs` (MODIFY)
  - Spec: `specs/search-command.md#acceptance-criteria`
  - Do: Add tests for keyword matching, regex matching, and empty results.
- [ ] 3.2 Integration test for search command
  - File: `tests/search_test.rs` (CREATE)
  - Spec: `specs/search-command.md#acceptance-criteria`
  - Do: Test the full flow from CLI invocation to output.