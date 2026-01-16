# Tasks

## 1. Data Layer

- [ ] 1.1 Define ArchivedChange struct
  - File: `src/cli/list.rs` (MODIFY)
  - Spec: `specs/list-changes.md#interfaces`
  - Do: Implement a struct to hold date, change_id, and summary.
  - Depends: none

## 2. Logic Layer

- [ ] 2.1 Implement archive folder name parsing
  - File: `src/cli/list.rs` (MODIFY)
  - Spec: `specs/list-changes.md#interfaces`
  - Do: Add `parse_archive_folder_name` and `format_date` functions.
  - Depends: 1.1

- [ ] 2.2 Implement summary extraction logic
  - File: `src/parser/markdown.rs` (MODIFY)
  - Spec: `specs/list-changes.md#requirements`
  - Do: Implement `extract_heading_section` to find a heading and capture the first paragraph with truncation.
  - Depends: none

- [ ] 2.3 Implement listing functions
  - File: `src/cli/list.rs` (MODIFY)
  - Spec: `specs/list-changes.md#interfaces`
  - Do: Implement `run` for active/brief list and `run_archived_detailed` for the table view.
  - Depends: 2.1, 2.2

## 3. Integration

- [ ] 3.1 Register CLI commands
  - File: `src/main.rs` (MODIFY)
  - Spec: `specs/list-changes.md#overview`
  - Do: Add `List` and `Archived` variants to `Commands` enum and wire them to `cli::list` functions.
  - Depends: 2.3

## 4. Testing

- [ ] 4.1 Unit tests for parsing and extraction
  - File: `src/cli/list.rs` (MODIFY)
  - Verify: `specs/list-changes.md#acceptance-criteria`
  - Do: Add tests for `parse_archive_folder_name` and `format_date`.
  - Depends: 2.1

- [ ] 4.2 Unit tests for markdown extraction
  - File: `src/parser/markdown.rs` (MODIFY)
  - Verify: `specs/list-changes.md#acceptance-criteria`
  - Do: Add tests for various markdown scenarios (missing heading, truncation, multiline).
  - Depends: 2.2
