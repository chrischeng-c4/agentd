# Changelog

All notable changes to Specter will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-12

### Added
- Initial release of Specter
- Core CLI commands: proposal, challenge, reproposal, implement, verify, archive
- Utility commands: init, list, status, refine
- AI orchestration via external scripts (Gemini, Codex, Claude)
- Interactive progress indicators
- Change management with phases (Proposed, Challenged, Implementing, Testing, Complete, Archived)
- Configuration system with `.specter/config.toml`
- Script runner with progress feedback
- Rust-based implementation for performance and reliability

### Key Features
- **Challenge Phase**: Codex-powered proposal review
- **Iterative Refinement**: proposal → challenge → reproposal loop
- **Cost-Effective**: 70-75% cost reduction vs pure Claude approach
- **Type-Safe**: Rust compile-time guarantees
- **Single Binary**: No runtime dependencies

[Unreleased]: https://github.com/your-repo/specter/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-repo/specter/releases/tag/v0.1.0
