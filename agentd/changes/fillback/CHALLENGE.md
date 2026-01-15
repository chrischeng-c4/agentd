# Challenge Report: fillback

## Summary
The proposal is directionally solid but has a couple of high-severity spec inconsistencies that need to be resolved before implementation. Once the CLI contract and artifact guarantees are clarified, the rest aligns with the existing Agentd structure.

## Internal Consistency Issues
These are HIGH priority - must fix before implementation.

### Issue: Conflicting CLI contract for `change_id`
- **Severity**: High
- **Category**: Consistency
- **Description**: The CLI syntax for `change_id` is inconsistent. The data model and tasks define `change_id` as positional, but the acceptance criteria uses a `--change-id` flag, creating ambiguity about the actual CLI interface.
- **Location**: `agentd/changes/fillback/specs/fillback_command.md` (Data Model vs Acceptance Criteria "Code Strategy Execution"), `agentd/changes/fillback/tasks.md` 1.1
- **Recommendation**: Choose one CLI contract (positional or flag) and update the acceptance criteria, data model, and tasks to match. Prefer positional to stay consistent with existing commands unless there is a strong reason to add a flag.

### Issue: Guaranteed artifact creation is underspecified in tasks
- **Severity**: High
- **Category**: Completeness
- **Description**: The spec and acceptance criteria require `proposal.md`, `tasks.md`, and at least one `specs/*.md` file to exist after any strategy runs, even if placeholders are used. The tasks only mention backfilling `proposal.md` and `tasks.md`, with no explicit step to ensure `specs/` and a placeholder spec file exist when strategies produce no spec output.
- **Location**: `agentd/changes/fillback/specs/fillback_command.md` (R3, Acceptance Criteria "Full Artifact Generation"), `agentd/changes/fillback/tasks.md` 1.3
- **Recommendation**: Add an explicit step to create `specs/` and a placeholder spec (e.g., `_skeleton.md`) when missing, ideally reusing the existing template loader/skeleton helpers.

## Code Alignment Issues

### Issue: "Orchestrator" integration does not map cleanly to current architecture
- **Severity**: Medium
- **Category**: Conflict
- **Description**: The tasks call for "Add Fillback support to Orchestrator," but the current integration points for AI workflows are in `ScriptRunner` rather than in an orchestrator abstraction (the orchestrator structs are placeholders). The proposal does not specify where the new fillback script hook will live in the actual code.
- **Location**: `src/orchestrator/script_runner.rs`, `src/orchestrator/mod.rs`, `src/orchestrator/gemini.rs`
- **Note**: No refactor/BREAKING intent noted.
- **Recommendation**: Define the integration point explicitly (e.g., add `run_gemini_fillback` to `ScriptRunner` and use it in `fillback` strategies), or formalize an orchestrator interface and update all existing commands consistently.

### Issue: Change structure validation expects `specs/` even if strategy outputs none
- **Severity**: Medium
- **Category**: Conflict
- **Description**: `Change::validate_structure` requires the `specs/` directory to exist. If a strategy fails to create it (or only writes proposal/tasks), downstream commands will fail validation even though the spec says placeholders are acceptable.
- **Location**: `src/models/change.rs`
- **Note**: No refactor/BREAKING intent noted.
- **Recommendation**: Ensure `fillback` always creates `specs/` and a placeholder spec, or update validation requirements if the workflow is meant to allow spec-less changes (not currently supported).

## Quality Suggestions

### Issue: Missing tests for strategy selection and artifact guarantees
- **Severity**: Low
- **Category**: Completeness
- **Description**: There are no planned tests for strategy auto-detection, artifact creation guarantees, or error handling (invalid strategy, missing path).
- **Recommendation**: Add unit tests for `StrategyFactory` detection and a simple integration test to verify `proposal.md`, `tasks.md`, and `specs/_skeleton.md` are created in an empty directory.

### Issue: Code scanning should guard against binary/large files
- **Severity**: Low
- **Category**: Other
- **Description**: The code strategy intends to read source files for LLM input. Without file size/binary checks, it risks scanning large or non-text files that slow execution or blow token limits.
- **Recommendation**: Add size and UTF-8/binary guards (and consider extension-based filters) when assembling `SpecGenerationRequest`.

## Verdict
- [ ] APPROVED - Ready for implementation
- [x] NEEDS_REVISION - Address issues above (HIGH severity)
- [ ] REJECTED - Fundamental problems, needs rethinking

**Next Steps**: Resolve the CLI contract inconsistency and add explicit spec-placeholder creation steps; then re-run the consistency check before implementation.
