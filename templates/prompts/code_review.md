---
task_type: code_review
agent: codex
phase: implement
variables:
  - change_id
  - iteration
mcp_instruction: |
  mcp__agentd-mcp__get_task(project_path="{{project_path}}", change_id="{{change_id}}", task_type="code_review")
---
# Task: Code Review (Iteration {{iteration}})

All agentd MCP tools require `project_path="{{project_path}}"`

## Change ID
{{change_id}}

## Instructions

1. **Get requirements**:
   - Read all specs and tasks

2. **Get implementation summary**:
   - Review changed files against requirements

3. **Review focus**:
   - Test results (are all tests passing?)
   - Security (any vulnerabilities?)
   - Best practices (performance, error handling)
   - Requirement compliance (does code match specs?)

4. **Submit review**:
   - Use `create_review` MCP tool with findings

## Severity Guidelines
- **HIGH**: Failing tests, security issues, missing features
- **MEDIUM**: Style issues, missing tests, minor improvements
- **LOW**: Suggestions, nice-to-haves

## Verdict Guidelines
- **APPROVED**: All tests pass, no HIGH issues
- **NEEDS_CHANGES**: Some issues exist (fixable)
- **MAJOR_ISSUES**: Critical problems

## MCP Tools

### Read Context
```
mcp__agentd-mcp__read_all_requirements(project_path="{{project_path}}", change_id="{{change_id}}")
mcp__agentd-mcp__list_changed_files(project_path="{{project_path}}", change_id="{{change_id}}")
```

### Generate Artifact
```
mcp__agentd-mcp__create_review(project_path="{{project_path}}", change_id="{{change_id}}", iteration={{iteration}}, test_results={status:"...", total:0, passed:0, failed:0, skipped:0}, security_status="CLEAN|WARNINGS|VULNERABILITIES|NOT_RUN", issues=[{severity:"HIGH|MEDIUM|LOW", title:"...", description:"..."}], verdict="APPROVED|NEEDS_CHANGES|MAJOR_ISSUES", next_steps="...")
```
