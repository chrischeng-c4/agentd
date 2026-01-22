# Change: fix-workflow-bugs

## Summary

This change fixes several critical workflow bugs: missing `tasks.md` frontmatter support, a missing `create_review` tool for implementation reviews, and fragile stdin pipe handling in LLM orchestrations.

## Why

These bugs block the full automation of the Agentd workflow:
- The absence of `tasks.md` frontmatter prevents sequential implementation from knowing the change context and layer order.
- Without a `create_review` tool, Codex cannot structure its review findings, leaving `REVIEW.md` as a useless template.
- Stdin pipe errors (Broken Pipe) cause intermittent failures in Claude/Codex orchestrations when prompts are large or processes close early.

## What Changes

- **Tasks Robustness**:
  - Update `TasksFrontmatter` to support `change_id` as an alias for `id` for better backward compatibility.
  - Modify `tasks_service.rs` to explicitly write `change_id` in generated frontmatter.
  - Enhance `TaskGraph` to handle missing frontmatter by inferring layers from markdown headings (e.g., "## 1. Data Layer").
  - Fix hardcoded `{{change_id}}` placeholder in `TaskGraph` spec path resolution.
- **Review Tooling**:
  - Implement `create_review` logic in `implementation_service.rs` to format structured findings into `REVIEW.md`.
  - Add `create_review` MCP tool to `mcp/tools/implementation.rs` and register it in the tool registry.
  - Update code review prompts to use the new `create_review` tool.
- **Orchestration Stability**:
  - Update `ScriptRunner` to gracefully handle `BrokenPipe` errors during stdin communication with LLM CLI tools.

## Impact

- Affected specs:
  - `agentd/changes/fix-workflow-bugs/specs/workflows.md`
  - `agentd/changes/fix-workflow-bugs/specs/tasks-robustness.md`
  - `agentd/changes/fix-workflow-bugs/specs/review-tool.md`
  - `agentd/changes/fix-workflow-bugs/specs/robust-orchestration.md`
- Affected code:
  - `src/models/frontmatter.rs`
  - `src/models/task_graph.rs`
  - `src/services/tasks_service.rs`
  - `src/services/implementation_service.rs`
  - `src/mcp/tools/implementation.rs`
  - `src/mcp/tools/mod.rs`
  - `src/orchestrator/script_runner.rs`
  - `templates/prompts/code_review.md`
- Breaking changes: No.