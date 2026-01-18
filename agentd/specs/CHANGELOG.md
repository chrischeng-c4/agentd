# Specs Changelog

All notable changes to specifications will be documented here.

## [Unreleased]

### Added
- `workflows.md`: High-level workflow specification for plan/impl/archive
  - Phase-only state machine design
  - Challenge updates phase based on verdict (APPROVED → challenged, REJECTED → rejected)
  - New `rejected` phase for fundamentally flawed proposals

## 2026-01-17: Add Plan Viewer UI (plan-viewer-ui)
Added a standalone UI window viewer for `agentd` plans using `wry`. This viewer provides a rich, interactive interface for reviewing proposals and challenges, featuring native Mermaid diagram rendering, STATE.yaml rendering, and support for human annotations.
- Related specs: plan-viewer.md, annotations.md

## 2026-01-17: Enhance Fillback Process (improve-fillback-2)
Added `fillback-enhancement.md` to specify the enhanced fillback process, transitioning from simple file-scanning to AST-based analysis and interactive clarification.
- Related specs: fillback-enhancement.md

## 2026-01-16: Add dedicated archived command (test-retry)
Added a dedicated `agentd archived` CLI command to improve discoverability of project history and provide a detailed view of completed changes. This allowed users to browse past work with richer context, including dates and extracted summaries.
- Related specs: archived-command.md
