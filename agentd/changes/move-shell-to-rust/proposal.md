# Change: move-shell-to-rust

## Summary

Move the AI integration logic currently implemented in shell scripts (`agentd/scripts/*.sh`) into the Rust codebase. This involves implementing the placeholder orchestrator structs (`GeminiOrchestrator`, `ClaudeOrchestrator`, `CodexOrchestrator`) and refactoring `ScriptRunner` to invoke CLI tools directly.

## Why

- **Reliability**: Shell scripts are brittle and harder to test than Rust code.
- **Portability**: Moving logic to Rust reduces dependence on Unix-specific shell features and external script files.
- **Maintainability**: Centralizing prompt logic in Rust makes it easier to version, refactor, and manage.
- **Better UX**: Rust allows for more granular progress reporting and better error handling than piping output from shell scripts.
- **Security**: Reduces the surface area for shell injection and simplifies permission management.

## What Changes

- **Implement Orchestrators**: Full implementation of `GeminiOrchestrator`, `ClaudeOrchestrator`, and `CodexOrchestrator` in `src/orchestrator/`.

- **Pre-processing Logic**: Move local tool execution (e.g., `cargo test`, `cargo audit`, `clippy`) from `codex-review.sh` into the Rust orchestrator layer.

- **ModelSelector Integration**: Wire `ModelSelector` into all orchestrators to automate model selection based on task complexity.

- **Prompt Generation**: Move all HEREDOC prompts from shell scripts into Rust (using constants or templates).

- **Direct CLI Execution**: Update orchestrators to use `tokio::process::Command` to call `gemini`, `claude`, and `codex` directly.

- **Refactor `ScriptRunner`**: Evolve `ScriptRunner` into a internal generic CLI runner used by orchestrators, removing its script-specific methods.

- **Clean up**: Remove legacy `.sh` scripts from `agentd/scripts/` and update `agentd init` to stop generating them.

- **Update CLI Commands**: Update ALL CLI commands (including `fillback`, `archive`, `resolve`, etc.) to use the new orchestrators.

- **Configuration**: Deprecate `scripts_dir` in `AgentdConfig` as core logic is now internal.



## Impact



- Affected specs: `specs/orchestrator.md` (to be created)

- Affected code:

    - `src/orchestrator/mod.rs`

    - `src/orchestrator/gemini.rs`

    - `src/orchestrator/claude.rs`

    - `src/orchestrator/codex.rs`

    - `src/orchestrator/script_runner.rs`

    - `src/cli/init.rs` (update to stop script generation)

    - `src/models/change.rs` (deprecate `scripts_dir`)

    - `src/cli/*.rs` (all commands)

    - `src/fillback/code.rs` (update to use orchestrator)

- Breaking changes: No change to user-facing CLI interface. `agentd/scripts/` will no longer be required or generated for core operation. Existing custom scripts in that directory will still work if called via the generic runner.
