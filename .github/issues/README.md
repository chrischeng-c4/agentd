# Local Issue Tracking

This directory contains issue descriptions that can be posted to GitHub or tracked locally.

## Current Issues

1. **tasks-format-inconsistency.md** - tasks.md format differs between sequential and non-sequential modes

## How to Create GitHub Issue

```bash
# Using GitHub CLI
gh issue create --title "tasks.md Format Inconsistency" \
  --body-file .github/issues/tasks-format-inconsistency.md \
  --label bug,consistency
```
