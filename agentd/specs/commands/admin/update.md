# Specification: Update Command

## Overview

The `update` command checks for newer versions of Agentd and performs a self-upgrade by downloading and installing the latest release from GitHub. It supports check-only mode to see if an update is available without installing.

## Requirements

### R1: Version Check
- Query GitHub API for latest release
- Compare with current installed version (CARGO_PKG_VERSION)
- Display "already up to date" if current version is latest

### R2: Self-Upgrade
- Download latest binary from GitHub releases
- Replace current binary with new version
- Maintain file permissions
- Rollback on failure

### R3: Check-Only Mode
- `--check` flag shows available update without installing
- Display current and latest versions
- Exit without performing upgrade

## Command Signature

```bash
agentd update [OPTIONS]
```

**Options:**
- `-c, --check`: Check for updates without installing

## Exit Codes

- `0`: Success (up to date or upgrade completed)
- `1`: Error (network failure, download error, permission denied)

## Examples

```bash
$ agentd update --check
Current version: 0.1.0
Latest version:  0.2.0
Update available!

$ agentd update
Downloading agentd 0.2.0...
Installing...
✅ Successfully upgraded to 0.2.0
```

## Notes

- Requires network access to GitHub
- May require sudo/admin rights depending on installation location
- Binary replacement is atomic - no partial states
- Old binary is not backed up automatically
