# Specs Changelog

All notable changes to specifications will be documented here.

## [Unreleased]

### Added
- `workflows.md`: High-level workflow specification for plan/impl/archive
  - Phase-only state machine design
  - Challenge updates phase based on verdict (APPROVED → challenged, REJECTED → rejected)
  - New `rejected` phase for fundamentally flawed proposals
