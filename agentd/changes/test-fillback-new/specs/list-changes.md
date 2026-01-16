# Specification: List Changes

## Overview

The `list` and `archived` commands provide visibility into the lifecycle of changes managed by Agentd. This feature encompasses listing active changes in the `agentd/changes` directory and archived changes in the `agentd/archive` directory.

## Requirements

### R1: Active Change Listing
The system must list all active changes by scanning the `agentd/changes` directory. Each directory found represents an active change ID.

### R2: Archived Change Listing (Brief)
The system must allow listing archived changes by scanning the `agentd/archive` directory when the `--archived` flag is used with the `list` command.

### R3: Detailed Archived Change View
The system must provide a detailed tabular view of archived changes via the `archived` command. This view includes the completion date, the change ID, and a brief summary.

### R4: Archived Folder Pattern
Archived changes must follow the naming convention `{YYYYMMDD}-{change_id}`. The system must parse this pattern to extract the date and the original change ID.

### R5: Summary Extraction
For the detailed view, the system must read the `proposal.md` file within each archived change and extract the first paragraph under the "Summary" heading. The summary should be truncated to 80 characters.

### R6: Temporal Ordering
Detailed archived changes must be displayed in descending chronological order (newest first).

## Interfaces

```
FUNCTION run(archived: bool) -> Result<()>
  INPUT: archived (flag to toggle between active and archived list)
  OUTPUT: Success or Error
  SIDE_EFFECTS: Prints list of change names to stdout

FUNCTION run_archived_detailed() -> Result<()>
  INPUT: None
  OUTPUT: Success or Error
  SIDE_EFFECTS: Prints detailed table of archived changes to stdout

FUNCTION parse_archive_folder_name(folder_name: String) -> Option<(String, String)>
  INPUT: folder_name (directory name in agentd/archive)
  OUTPUT: Some((date_str, change_id)) or None if malformed

FUNCTION format_date(date_str: String) -> String
  INPUT: YYYYMMDD string
  OUTPUT: YYYY-MM-DD string
```

## Acceptance Criteria

### Scenario: List Active Changes
- **WHEN** the user runs `agentd list`
- **THEN** the system prints a list of all directory names found in `agentd/changes`

### Scenario: List Archived Changes (Detailed)
- **WHEN** the user runs `agentd archived` with folders `20260101-feat-1` and `20260115-feat-2` in `agentd/archive`
- **THEN** the system displays a table sorted with `2026-01-15` first, then `2026-01-01`
- **AND** the "ID" column shows `feat-2` and `feat-1` respectively

### Scenario: Summary Extraction and Truncation
- **WHEN** a `proposal.md` has a 100-character summary under `## Summary`
- **THEN** the system extracts the text and truncates it to 77 characters followed by "..."
- **AND** the extraction stops at the next heading or empty line

### Scenario: Skip Malformed Archive Folders
- **WHEN** a folder in `agentd/archive` does not match the `{YYYYMMDD}-{change_id}` pattern
- **THEN** the system prints a warning to stderr and continues processing other folders
