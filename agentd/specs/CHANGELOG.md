# Specs Changelog

All notable changes to specifications will be documented here.

## [Unreleased]

### Added
- `workflows.md`: High-level workflow specification for plan/impl/archive
  - Phase-only state machine design
  - Challenge updates phase based on verdict (APPROVED → challenged, REJECTED → rejected)
  - New `rejected` phase for fundamentally flawed proposals

## 2026-01-16: Add dedicated archived command (test-retry)
Added a dedicated `agentd archived` CLI command to improve discoverability of project history and provide a detailed view of completed changes. This allowed users to browse past work with richer context, including dates and extracted summaries.
- Related specs: archived-command.md
