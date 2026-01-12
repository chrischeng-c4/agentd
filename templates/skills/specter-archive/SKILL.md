---
name: specter-archive
description: Archive completed change
user-invocable: true
---

# /specter:archive

Archive completed and verified change.

## Usage

```bash
specter archive <change-id>
```

## Example

```bash
specter archive add-oauth
```

## What it does

1. Applies spec deltas to main `specs/` directory
2. Moves change to `changes/archive/YYYY-MM-DD-<change-id>/`
3. Updates `CHANGELOG.md`
4. Shows archive location

## Result

Change is now archived and specs are updated.
