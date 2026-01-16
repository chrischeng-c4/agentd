# Tasks

## 1. Data Layer

- [ ] 1.1 Add heading extraction logic
  - File: `src/parser/markdown.rs` (MODIFY)
  - Spec: `specs/archived-command.md#interfaces`
  - Do: Implement `extract_heading_section` to retrieve the first paragraph under a specified markdown heading, with 80-char truncation.
  - Depends: none

## 2. Logic Layer

- [ ] 2.1 Enhance archived listing logic
  - File: `src/cli/list.rs` (MODIFY)
  - Spec: `specs/archived-command.md#interfaces`
  - Do: Implement `run_archived_detailed`. Iterate through archived folders, skip malformed ones, extract summaries using the new parser helper, and handle the case where the archive directory is empty or missing.
  - Depends: 1.1

- [ ] 2.2 Format archived output
  - File: `src/cli/list.rs` (MODIFY)
  - Spec: `specs/archived-command.md#formatted-output`
  - Do: Implement table formatting for the collected archived changes (Date, ID, Summary).
  - Depends: 2.1

## 3. Integration

- [ ] 3.1 Register `archived` command
  - File: `src/main.rs` (MODIFY)
  - Spec: `specs/archived-command.md#command-discovery`
  - Do: Add `Archived` to the `Commands` enum and map it to `agentd::cli::list::run_archived_detailed()`.
  - Depends: 2.2

- [ ] 3.2 Update `list` command help
  - File: `src/main.rs` (MODIFY)
  - Spec: `specs/archived-command.md#command-discovery`
  - Do: Update the doc comment for the `List` command to mention the new `archived` command for a detailed view.
  - Depends: 3.1

## 4. Testing

- [ ] 4.1 Unit tests for summary extraction
  - File: `src/parser/markdown.rs` (MODIFY)
  - Verify: `specs/archived-command.md#acceptance-criteria`
  - Do: Add unit tests for `extract_heading_section` covering success, missing heading, and truncation.
  - Depends: 1.1

- [ ] 4.2 Unit tests for archived listing
  - File: `src/cli/list.rs` (MODIFY)
  - Verify: `specs/archived-command.md#acceptance-criteria`
  - Do: Add unit tests for the archive directory parsing logic, including empty and malformed folder cases.
  - Depends: 3.1
