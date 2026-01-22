# Implementation Notes: fix-workflow-bugs

## Summary

Fixed critical workflow bugs that blocked the full automation of Agentd workflows:
- TaskGraph now gracefully handles missing frontmatter
- Spec paths are correctly resolved with actual change_id
- BrokenPipe errors are handled non-fatally
- New `create_review` MCP tool for structured review submission

## Files Modified

### src/services/implementation_service.rs
- Added `create_review` function with structured types (Severity, ReviewVerdict, ReviewIssue)
- Generates properly formatted REVIEW.md from structured input

### src/mcp/tools/implementation.rs
- Added `create_review_definition()` tool definition
- Added `execute_create_review()` handler for MCP calls

### src/mcp/tools/mod.rs
- Registered `create_review` tool in all_tools_vec and review_tools
- Added call_tool handler for "create_review"

### src/models/task_graph.rs
- `parse_frontmatter()` now returns default on parse failure (robustness)
- `build_layers()` now infers layers from task IDs when frontmatter lacks layer definitions
- `group_by_spec()` now accepts change_id parameter
- Spec paths now use actual change_id instead of literal `{{change_id}}`

### src/orchestrator/script_runner.rs
- BrokenPipe errors during stdin write are now handled gracefully
- Process closing stdin early no longer causes fatal errors

### src/models/frontmatter.rs
- TasksFrontmatter.id now has `#[serde(alias = "change_id")]` for backward compatibility

### src/services/tasks_service.rs
- tasks.md frontmatter now emits both `id` and `change_id` fields

### templates/prompts/code_review.md
- Updated instructions to use `create_review` MCP tool
- Added structured input format for review submission

## Requirements Implemented

| Issue | Severity | Status | Fix |
|-------|----------|--------|-----|
| Missing create_review | HIGH | ✅ | New MCP tool and service |
| TaskGraph requires frontmatter | HIGH | ✅ | Graceful fallback to defaults |
| Hardcoded {{change_id}} placeholder | HIGH | ✅ | Dynamic change_id from path |
| BrokenPipe fatal error | HIGH | ✅ | Non-fatal EPIPE handling |
| Missing change_id alias | MEDIUM | ✅ | Serde alias attribute |
| Review prompt outdated | MEDIUM | ✅ | Updated to use create_review |

## Testing

All 401 tests pass after changes.
