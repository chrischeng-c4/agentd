---
task_type: code_review
agent: codex
phase: implement
variables:
  - change_id
  - iteration
---
# Task: Code Review (Iteration {{iteration}})

## Change ID
{{change_id}}

## Instructions

1. **Get requirements**:
   - Use: `read_all_requirements` with change_id="{{change_id}}"

2. **Get implementation summary**:
   - Use: `list_changed_files` with change_id="{{change_id}}"

3. **Review focus**:
   - Test results (are all tests passing?)
   - Security (any vulnerabilities?)
   - Best practices (performance, error handling)
   - Requirement compliance (does code match specs?)

4. **Submit review** using the `create_review` MCP tool:
   - `change_id`: "{{change_id}}"
   - `iteration`: {{iteration}}
   - `test_results`: { status, total, passed, failed, skipped }
   - `security_status`: "CLEAN" | "WARNINGS" | "VULNERABILITIES" | "NOT_RUN"
   - `issues`: Array of { severity, title, description, file_path?, line_number?, recommendation? }
   - `verdict`: "APPROVED" | "NEEDS_CHANGES" | "MAJOR_ISSUES"
   - `next_steps`: Optional string describing what to do next

## Severity Guidelines
- **HIGH**: Failing tests, security issues, missing features
- **MEDIUM**: Style issues, missing tests, minor improvements
- **LOW**: Suggestions, nice-to-haves

## Verdict Guidelines
- **APPROVED**: All tests pass, no HIGH issues
- **NEEDS_CHANGES**: Some issues exist (fixable)
- **MAJOR_ISSUES**: Critical problems

## Tools to Use
- `read_all_requirements`, `list_changed_files` (required for reading)
- `create_review` (required for submitting results)
