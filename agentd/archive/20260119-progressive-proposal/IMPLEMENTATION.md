# Implementation Notes: Progressive Proposal

## Summary

This implementation adds a **progressive proposal** workflow to agentd, enabling:
1. Session ID tracking for deterministic resume-by-index
2. Self-review step after proposal generation
3. Resume-by-index for all Gemini CLI calls (instead of `--resume latest`)

## Changes Made

### 1. Data Model Updates (`src/models/frontmatter.rs`)
- Added `session_id: Option<String>` field to `State` struct
- Updated `Default` impl to include `session_id: None`

### 2. State Schema (`agentd/schemas/state.schema.json`)
- Added `session_id` field definition with type `["string", "null"]`

### 3. State Manager (`src/state/manager.rs`)
- Added `set_session_id()` setter method
- Added `session_id()` getter method
- Updated `StateManager::load()` to include `session_id` in new state initialization

### 4. Usage Metrics (`src/orchestrator/script_runner.rs`)
- Added `session_id: Option<String>` field to `UsageMetrics`
- Added `GeminiInitResponse` struct to parse `{"type":"init","session_id":"..."}` messages
- Updated `parse_gemini_usage()` to extract session_id from init messages
- Added `run_llm_with_cwd()` method to support working directory for Gemini

### 5. CLI Mapper (`src/orchestrator/cli_mapper.rs`)
- Added `ResumeMode` enum with variants: `None`, `Latest`, `ByIndex(u32)`
- Added `build_args_with_resume()` method that accepts `ResumeMode`
- Updated `build_args()` to delegate to `build_args_with_resume()`
- Gemini `--resume <index>` format supported for `ByIndex` mode

### 6. Prompts (`src/orchestrator/prompts.rs`)
- Added `proposal_self_review_prompt()` function
- Prompts Gemini to review generated files and output markers:
  - `<review>PASS</review>` - no issues found
  - `<review>NEEDS_REVISION</review>` - issues found and fixed

### 7. Gemini Orchestrator (`src/orchestrator/gemini.rs`)
- Added `SelfReviewResult` enum (Pass, NeedsRevision)
- Added `detect_self_review_marker()` function to parse output
- Added `find_session_index()` async function to look up session by UUID
- Added `run_self_review()` method for self-review step
- Added `run_reproposal_with_session()` method for resume-by-index
- Updated all methods to use `run_llm_with_cwd()` with project_root

### 8. Module Exports (`src/orchestrator/mod.rs`)
- Exported `ResumeMode`, `SelfReviewResult`, `detect_self_review_marker`, `find_session_index`

### 9. Proposal CLI (`src/cli/proposal.rs`)
- Updated `run_proposal_step()` to:
  - Save session_id to STATE.yaml after generation
  - Run self-review loop (max 3 iterations) after proposal
  - Log self-review outcomes (Pass/NeedsRevision)
- Updated `run_reproposal_step()` to use resume-by-index when session_id is available

### 10. Reproposal CLI (`src/cli/reproposal.rs`)
- Updated standalone reproposal command to use resume-by-index when session_id is available

## Tests Added

### Self-Review Marker Detection (9 tests in `gemini.rs`)
- `test_detect_self_review_marker_pass`
- `test_detect_self_review_marker_needs_revision`
- `test_detect_self_review_marker_in_raw_output`
- `test_detect_self_review_marker_multiline`
- `test_detect_self_review_marker_array_content`
- `test_detect_self_review_marker_no_marker_returns_pass`
- `test_detect_self_review_marker_empty_output`
- `test_detect_self_review_marker_escaped_characters`
- `test_detect_self_review_prefers_needs_revision`

### Session Index Lookup (2 tests in `gemini.rs`)
- `test_session_list_regex_parsing`
- `test_session_list_no_match_for_invalid_lines`

### CLI Mapper Resume-by-Index (7 tests in `cli_mapper.rs`)
- `test_gemini_args_resume_by_index`
- `test_gemini_args_resume_mode_none`
- `test_gemini_args_resume_mode_latest`
- `test_codex_ignores_resume_by_index`
- `test_claude_ignores_resume_by_index`
- `test_build_args_delegates_to_build_args_with_resume`

### Session ID Extraction (4 tests in `script_runner.rs`)
- `test_parse_gemini_usage_extracts_session_id`
- `test_parse_gemini_usage_no_init_message`
- `test_parse_gemini_usage_with_uuid_format_session_id`
- Updated `test_parse_gemini_usage_no_metadata` to check session_id

### Session ID Persistence (5 tests in `manager.rs`)
- `test_session_id_setter_and_getter`
- `test_session_id_persistence`
- `test_session_id_in_yaml_serialization`
- `test_session_id_null_handling`
- `test_session_id_marks_dirty`

### Prompt Tests (1 test in `prompts.rs`)
- `test_proposal_self_review_prompt`

## Workflow Changes

### Before
```
proposal → validate → challenge → [reproposal loop] → done
                      (--resume latest)
```

### After
```
proposal → self-review loop → validate → challenge → [reproposal loop] → done
           (max 3 iterations)           (--resume <index>)
```

The self-review step allows Gemini to catch and fix obvious issues before the validation step, reducing iteration cycles.

## Review Fixes (Iteration 1)

The following issues from code review were addressed:

### HIGH: Resume-by-index failures now exit instead of fallback
- **Issue**: When session lookup failed, the workflow silently fell back to `--resume latest`
- **Fix**: Both `proposal.rs` and `reproposal.rs` now exit with status 1 when:
  - Session ID capture fails: "Failed to capture session ID"
  - Session lookup fails: "Session not found, please re-run proposal"
  - STATE.yaml loading fails: "Failed to load STATE.yaml"

### HIGH: Non-zero exit on failure implemented
- **Issue**: Workflow returned `Ok(())` on failures, making them undetectable in CI/scripts
- **Fix**: `proposal.rs` now calls `std::process::exit(1)` for:
  - Format validation failure after max iterations
  - Challenge `NEEDS_REVISION` after max iterations
  - Challenge `REJECTED` verdict
  - Challenge `UNKNOWN` verdict

### MEDIUM: Self-review logging now matches spec
- **Issue**: Logs used informal messages ("✅ Self-review passed")
- **Fix**: Updated to spec-compliant messages:
  - `Self-review: PASS (no changes)`
  - `Self-review: NEEDS_REVISION (files updated)`

### MEDIUM: Added tests for session lookup failure paths
- **Issue**: Tests only covered regex parsing, not error handling
- **Fix**: Added 5 new tests in `gemini.rs`:
  - `test_parse_session_list_uuid_found`
  - `test_parse_session_list_uuid_not_found`
  - `test_parse_session_list_unexpected_format_no_header`
  - `test_parse_session_list_malformed_lines`
  - `test_parse_session_list_empty_output`

## All Tests Pass

```
cargo test --lib
...
test result: ok. 251 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
