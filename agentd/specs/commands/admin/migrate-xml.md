# Specification: Migrate-XML Command

## Overview

The `migrate-xml` command migrates proposal files and challenge reviews from the old format to the new XML-wrapped format. It can migrate a single change or all changes in the project.

## Requirements

### R1: Format Detection
- Detect old format: Standalone CHALLENGE.md
- Detect new format: XML-wrapped review in proposal.md
- Skip already migrated changes

### R2: Single Change Migration
- Migrate specified change_id if provided
- Read CHALLENGE.md content
- Wrap in XML review block
- Append to proposal.md
- Archive old CHALLENGE.md

### R3: Batch Migration
- Migrate all changes if no change_id specified
- Scan agentd/changes/ directory
- Process each change independently
- Report success/failure per change

### R4: XML Wrapping Logic
- Wrap challenge content in `<review>` tags
- Include date attribute
- Preserve all markdown formatting
- Maintain verdict and issues structure

## Command Signature

```bash
agentd migrate-xml [change_id]
```

**Arguments:**
- `change_id` (optional): Specific change to migrate (default: all changes)

## Exit Codes

- `0`: Success (migrations completed)
- `1`: Error (file I/O error, invalid format)

## Examples

```bash
$ agentd migrate-xml feat-auth
Migrating feat-auth...
✓ Migrated CHALLENGE.md → proposal.md (XML format)

$ agentd migrate-xml
Migrating all changes...
✓ feat-auth
✓ feat-api
⊘ feat-login (already migrated)
```

## Notes

- Introduced in recent version to support new XML format
- Old CHALLENGE.md is preserved (not deleted) for safety
- Migration is idempotent - safe to run multiple times
- Part of gradual migration strategy from old to new format
