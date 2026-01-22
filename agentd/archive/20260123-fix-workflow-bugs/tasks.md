---
id: fix-workflow-bugs
type: tasks
version: 1
---

# Tasks

<meta>
  <purpose>Implementation tickets derived from specs</purpose>
  <constraint>NO actual code - just file paths, actions, and references</constraint>
</meta>

## 1. Data Layer

- [ ] 1.1 Support `change_id` alias in `TasksFrontmatter`
  - File: `src/models/frontmatter.rs` (MODIFY)
  - Spec: `specs/tasks-robustness.md#data-model`
  - Do: Add `#[serde(alias = "change_id")]` to the `id` field of `TasksFrontmatter` struct to support backward compatibility.
  - Depends: none

## 2. Logic Layer

- [ ] 2.1 Implement heading-based layer inference in `TaskGraph`
  - File: `src/models/task_graph.rs` (MODIFY)
  - Spec: `specs/tasks-robustness.md#r3-inferred-layers-from-headings`
  - Do: Update `parse_frontmatter` to return a default if missing, and `build_layers` to infer layer definitions from "## N. Name" headings if `frontmatter.layers` is empty.
  - Depends: 1.1

- [ ] 2.2 Fix hardcoded spec path placeholders in `TaskGraph`
  - File: `src/models/task_graph.rs` (MODIFY)
  - Spec: `specs/tasks-robustness.md#r2-resolve-hardcoded-spec-paths`
  - Do: In `group_by_spec`, replace the hardcoded `{{change_id}}` string with a dynamic value derived from the frontmatter or directory structure.
  - Depends: none

- [ ] 2.3 Update `TasksService` to generate dual ID frontmatter
  - File: `src/services/tasks_service.rs` (MODIFY)
  - Spec: `specs/tasks-robustness.md#r4-mandatory-frontmatter-in-generation`
  - Do: Update the tasks generation logic to explicitly write both `id` and `change_id` fields in the YAML frontmatter.
  - Depends: 1.1

- [ ] 2.4 Implement `create_review` service logic
  - File: `src/services/implementation_service.rs` (MODIFY)
  - Spec: `specs/review-tool.md#interfaces`
  - Do: Add `create_review` method that formats structured findings into a `REVIEW.md` file using the `format_review_markdown` helper.
  - Depends: none

- [ ] 2.5 Robust Stdin Orchestration in `ScriptRunner`
  - File: `src/orchestrator/script_runner.rs` (MODIFY)
  - Spec: `specs/robust-orchestration.md#interfaces`
  - Do: Update `write_to_stdin` to catch `BrokenPipe` (EPIPE) errors during `write_all` and ignore them if the process hasn't crashed, ensuring EOF is still signalled.
  - Depends: none

- [ ] 2.6 Sequential Workflow Enforcement in `ProposalEngine`
  - File: `src/cli/proposal_engine.rs` (MODIFY)
  - Spec: `specs/mcp-spec-tool.md#r3-sequential-workflow-enforcement`
  - Do: Add validation checks between the proposal, spec, and task generation phases to ensure the output of each phase exists and is non-empty before proceeding.
  - Depends: none

## 3. Integration

- [ ] 3.1 Implement `create_review` MCP tool
  - File: `src/mcp/tools/implementation.rs` (MODIFY)
  - Spec: `specs/review-tool.md#r1-structured-review-submission`
  - Do: Add a new MCP tool `create_review` that accepts the structured review payload and calls the implementation service.
  - Depends: 2.4

- [ ] 3.2 Register `create_review` tool in registry
  - File: `src/mcp/tools/mod.rs` (MODIFY)
  - Spec: `specs/review-tool.md#overview`
  - Do: Export and register the `create_review` tool so it's available to the implementation agent.
  - Depends: 3.1

- [ ] 3.3 Update code review instructions prompt
  - File: `templates/prompts/code_review.md` (MODIFY)
  - Spec: `specs/review-tool.md#overview`
  - Do: Update the "Instructions" and "Tools to Use" sections to direct Codex to use the `create_review` tool for submitting results.
  - Depends: 3.2

- [ ] 3.4 Enhanced Input Validation for Spec MCP tool
  - File: `src/mcp/tools/spec.rs` (MODIFY)
  - Spec: `specs/mcp-spec-tool.md#r4-validation-of-tool-inputs`
  - Do: Add explicit checks for `overview` length and `spec_id` format in the `execute` function, returning clear error messages if validation fails.
  - Depends: none

- [ ] 3.5 Enhanced Input Validation for Tasks MCP tool
  - File: `src/mcp/tools/tasks.rs` (MODIFY)
  - Spec: `specs/mcp-spec-tool.md#r4-validation-of-tool-inputs`
  - Do: Add explicit validation to ensure the `tasks` array is not empty and that each task has a valid layer name and number.
  - Depends: none

## 4. Testing

- [ ] 4.1 Test frontmatter alias and layer inference
  - File: `tests/tasks_test.rs` (CREATE)
  - Verify: `specs/tasks-robustness.md#acceptance-criteria`
  - Do: Add tests for `TasksFrontmatter` deserialization with `change_id` and `TaskGraph` construction from markdown with missing frontmatter.
  - Depends: 2.1

- [ ] 4.2 Test structured review generation
  - File: `tests/review_test.rs` (CREATE)
  - Verify: `specs/review-tool.md#acceptance-criteria`
  - Do: Add integration tests verifying that `create_review` correctly formats `REVIEW.md` and handles all verdict types.
  - Depends: 2.4

- [ ] 4.3 Test orchestration stdin resilience
  - File: `tests/orchestration_test.rs` (CREATE)
  - Verify: `specs/robust-orchestration.md#acceptance-criteria`
  - Do: Verify `ScriptRunner` behavior when child process closes stdin early, ensuring it continues to process stdout correctly.
  - Depends: 2.5