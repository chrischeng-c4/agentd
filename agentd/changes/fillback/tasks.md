# Tasks

## 1. CLI Layer
- [x] 1.1 Add `Fillback` command to `Commands` enum in `src/main.rs`
  - File: `src/main.rs` (MODIFY)
  - Spec: `specs/fillback_command.md#Data Model`
  - Do: Add `Fillback { change_id: String, #[arg(short, long)] path: Option<String>, #[arg(short, long)] strategy: Option<String> }` variant.
- [x] 1.2 Register module in `src/cli/mod.rs`
  - File: `src/cli/mod.rs` (MODIFY)
  - Do: Add `pub mod fillback;`.
- [x] 1.3 Implement command handler in `src/cli/fillback.rs`
  - File: `src/cli/fillback.rs` (CREATE)
  - Spec: `specs/fillback_command.md#Interfaces`
  - Do: Implement `async fn run` that:
    1. Resolves `change_id` conflicts using `crate::context::resolve_change_id_conflict`.
    2. Parses args and uses `StrategyFactory` to get the strategy.
    3. Executes the strategy.
    4. Checks for `proposal.md` and `tasks.md` in the change dir; writes default skeletons if missing.
- [x] 1.4 Update Command Dispatch
  - File: `src/main.rs` (MODIFY)
  - Do: Add match arm for `Commands::Fillback` to call `agentd::cli::fillback::run`.
- [x] 1.5 Add Dependencies
  - File: `Cargo.toml` (MODIFY)
  - Do: Add `ignore` crate for gitignore-aware file walking.

## 2. Logic Layer (Strategies)
- [x] 2.1 Create `src/fillback` module structure
  - File: `src/lib.rs` (MODIFY) - Add `pub mod fillback;`
  - File: `src/fillback/mod.rs` (CREATE) - Export submodules.
- [x] 2.2 Define `ImportStrategy` trait
  - File: `src/fillback/strategy.rs` (CREATE)
  - Spec: `specs/interfaces.md#R1: ImportStrategy Trait`
- [x] 2.3 Implement `OpenSpecStrategy`
  - File: `src/fillback/openspec.rs` (CREATE)
  - Spec: `specs/interfaces.md#R2: OpenSpec Strategy`
  - Do: Implement YAML/JSON parsing and conversion to Agentd models.
- [x] 2.4 Implement `SpeckitStrategy`
  - File: `src/fillback/speckit.rs` (CREATE)
  - Spec: `specs/interfaces.md#R3: Speckit Strategy`
  - Do: Implement Markdown parsing.
- [x] 2.5 Define `SpecGenerationRequest` model
  - File: `src/models/spec_generation.rs` (CREATE)
  - Spec: `specs/interfaces.md#R5: Spec Generation Request`
  - Do: Create the struct for orchestrator communication.
  - File: `src/models/mod.rs` (MODIFY) - Register module.
- [x] 2.6 Implement `CodeStrategy`
  - File: `src/fillback/code.rs` (CREATE)
  - Spec: `specs/interfaces.md#R4: Code Strategy`
  - Do: Implement file scanning using `ignore::Walk`. Construct `SpecGenerationRequest` and call Orchestrator.
- [x] 2.7 Implement `StrategyFactory`
  - File: `src/fillback/factory.rs` (CREATE)
  - Spec: `specs/fillback_command.md#R2: Strategy Selection`
  - Do: Implement `create` method with auto-detection logic (checking file existence/types).

## 3. Orchestration & Prompts
- [x] 3.1 Create `gemini-fillback.sh` script
  - File: `agentd/scripts/gemini-fillback.sh` (CREATE)
  - Do: Script to invoke Gemini with "Reverse Engineer" prompt.
- [x] 3.2 Create `fillback.toml` command template
  - File: `templates/gemini/commands/agentd/fillback.toml` (CREATE)
  - Do: Define the `agentd:fillback` command configuration for Gemini.
- [x] 3.3 Add `Fillback` support to Orchestrator
  - File: `src/orchestrator/mod.rs` (MODIFY)
  - Do: Add method/support for calling the fillback script/prompt.
- [x] 3.4 Update Init Flow
  - File: `src/cli/init.rs` (MODIFY)
  - Do: Add `gemini-fillback.sh` to script list and `fillback.toml` to command templates in `install` and `upgrade` functions.
