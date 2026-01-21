/// Prompt templates for AI tool orchestration
///
/// This module contains all prompt templates used by orchestrators to interact
/// with AI tools (Gemini, Claude, Codex). Prompts are parameterized and can be
/// customized based on change ID, description, and other context.
/// Generate Gemini proposal prompt
pub fn gemini_proposal_prompt(change_id: &str, description: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## User Request
{description}

## Instructions

**IMPORTANT**: You MUST use the CLI workflow to generate files. Do NOT output markdown directly.

1. Analyze the codebase and understand the request
2. Create proposal JSON with structure:
   {{
     "summary": "Brief 1-sentence description",
     "why": "Detailed motivation (min 50 chars)",
     "what_changes": ["Change 1", "Change 2"],
     "impact": {{
       "scope": "patch|minor|major",
       "affected_files": <number>,
       "affected_specs": ["spec-id"],
       "affected_code": ["path/to/code/"],
       "breaking_changes": null
     }}
   }}
3. Write JSON to /tmp/proposal-{change_id}.json using heredoc
4. Run: agentd proposal create {change_id} --json-file /tmp/proposal-{change_id}.json
5. Verify success by checking command output

Direct file creation or markdown output is NOT allowed. Use CLI workflow only.
"#,
        change_id = change_id,
        description = description
    )
}

/// Generate Gemini proposal prompt using CLI workflow (sequential generation)
pub fn gemini_proposal_with_mcp_prompt(change_id: &str, description: &str) -> String {
    format!(
        r#"## Task: Create proposal.md

Use the CLI workflow to generate proposal.md for this change.

## Change ID
{change_id}

## User Request
{description}

## Instructions

1. **Analyze the codebase** using your 2M context window:
   - Read project structure, existing code, patterns
   - Understand the technical landscape
   - Identify affected areas

2. **Determine required specs**:
   - What major components/features need detailed design?
   - List them in the `affected_specs` field
   - Use clear, descriptive IDs (e.g., `auth-flow`, `user-model`, `api-endpoints`)

3. **Create proposal JSON** with this structure:
   ```json
   {{
     "summary": "Brief 1-sentence description",
     "why": "Detailed business/technical motivation (min 50 chars)",
     "what_changes": [
       "Add X to handle Y",
       "Modify Z to support W"
     ],
     "impact": {{
       "scope": "patch|minor|major",
       "affected_files": <estimated number>,
       "affected_specs": [
         "spec-id-1",
         "spec-id-2"
       ],
       "affected_code": [
         "path/to/component/",
         "path/to/module/"
       ],
       "breaking_changes": null or "description"
     }}
   }}
   ```

4. **Write JSON to file** using heredoc:
   ```bash
   cat > /tmp/proposal-{change_id}.json <<'EOF'
   {{ JSON content here }}
   EOF
   ```

5. **Run CLI command**:
   ```bash
   agentd proposal create {change_id} --json-file /tmp/proposal-{change_id}.json
   ```

6. **Verify success** by checking command output

IMPORTANT: The `affected_specs` list determines which specs will be generated next. Be thorough but focused.
"#,
        change_id = change_id,
        description = description
    )
}

/// Generate Gemini spec prompt using CLI workflow (sequential generation)
pub fn gemini_spec_with_mcp_prompt(change_id: &str, spec_id: &str, context_files: &[String]) -> String {
    let context_list = if context_files.is_empty() {
        String::from("- agentd/changes/{change_id}/proposal.md")
    } else {
        context_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
    };

    format!(
        r#"## Task: Create spec '{spec_id}'

Use the CLI workflow to generate specs/{spec_id}.md for this change.

## Context Files (read these first):
{context_list}

## Instructions

1. **Read context files**:
   - Use: agentd file read {change_id} proposal
   - Use: agentd spec list {change_id} (to see existing specs)
   - Read existing specs to maintain consistency and avoid duplication

2. **Design this spec**:
   - Define clear, testable requirements (R1, R2, ...)
   - Add Mermaid diagrams if helpful:
     - Flow: sequenceDiagram for interactions
     - State: stateDiagram-v2 for state machines
     - Data Model: JSON Schema for data structures
   - Write acceptance scenarios (WHEN/THEN format)
   - Ensure consistency with proposal.md and other specs

3. **Create spec JSON** with this structure:
   ```json
   {{
     "title": "Human-readable title",
     "overview": "What this spec covers and why (min 50 chars)",
     "requirements": [
       {{
         "id": "R1",
         "title": "Short title",
         "description": "Detailed requirement description",
         "priority": "high|medium|low"
       }}
     ],
     "scenarios": [
       {{
         "name": "Happy path scenario",
         "given": "Optional precondition",
         "when": "Trigger condition with specific values",
         "then": "Expected outcome"
       }},
       {{
         "name": "Error case scenario",
         "when": "Error condition",
         "then": "Error handling behavior"
       }}
     ],
     "flow_diagram": "sequenceDiagram\\n    ...",
     "data_model": {{ JSON Schema object }}
   }}
   ```

4. **Write JSON to file** using heredoc:
   ```bash
   cat > /tmp/spec-{change_id}-{spec_id}.json <<'EOF'
   {{ JSON content here }}
   EOF
   ```

5. **Run CLI command**:
   ```bash
   agentd spec create {change_id} {spec_id} --json-file /tmp/spec-{change_id}-{spec_id}.json
   ```

6. **Verify success** by checking command output

IMPORTANT:
- Minimum 3 scenarios required (happy path + error cases + edge cases)
- Use specific values in scenarios, not placeholders
- Maintain consistency with other specs (no duplicate definitions)
"#,
        change_id = change_id,
        spec_id = spec_id,
        context_list = context_list
    )
}

/// Generate Gemini tasks prompt using CLI workflow (sequential generation)
pub fn gemini_tasks_with_mcp_prompt(change_id: &str, all_files: &[String]) -> String {
    let context_list = all_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n");

    format!(
        r#"## Task: Create tasks.md

Use the CLI workflow to generate tasks.md for this change.

## Context Files (read all):
{context_list}

## Instructions

1. **Read all context files**:
   - Use: agentd file read {change_id} proposal
   - Use: agentd spec list {change_id} (to list all specs)
   - Read all specs/*.md for detailed requirements

2. **Break down into tasks**:
   - Organize by layer (build order):
     - **data**: Database schemas, models, data structures
     - **logic**: Business logic, algorithms, core functionality
     - **integration**: API endpoints, external integrations
     - **testing**: Unit tests, integration tests
   - Each task should reference a spec requirement (e.g., `auth-flow:R1`)
   - Define file actions: CREATE, MODIFY, or DELETE
   - Set dependencies between tasks (e.g., task 2.1 depends on ["1.1"])

3. **Create tasks JSON** with this structure:
   ```json
   {{
     "tasks": [
       {{
         "layer": "data",
         "number": 1,
         "title": "Create User model",
         "file": {{
           "path": "src/models/user.rs",
           "action": "CREATE"
         }},
         "spec_ref": "user-model:R1",
         "description": "Detailed task description",
         "depends": []
       }},
       {{
         "layer": "logic",
         "number": 1,
         "title": "Implement OAuth flow",
         "file": {{
           "path": "src/auth/oauth.rs",
           "action": "CREATE"
         }},
         "spec_ref": "auth-flow:R1",
         "description": "Detailed task description",
         "depends": ["1.1"]
       }}
     ]
   }}
   ```

4. **Write JSON to file** using heredoc:
   ```bash
   cat > /tmp/tasks-{change_id}.json <<'EOF'
   {{ JSON content here }}
   EOF
   ```

5. **Run CLI command**:
   ```bash
   agentd tasks create {change_id} --json-file /tmp/tasks-{change_id}.json
   ```

6. **Verify success** by checking command output

IMPORTANT:
- All spec requirements must be covered by tasks
- Dependencies must be correct (no circular deps)
- File paths must be specific and accurate
- Layer organization must be logical (data → logic → integration → testing)
"#,
        change_id = change_id,
        context_list = context_list
    )
}

/// Generate self-review prompt for proposal.md (sequential generation)
pub fn proposal_self_review_with_mcp_prompt(change_id: &str) -> String {
    format!(
        r#"## Task: Review proposal.md

Read and review: agentd/changes/{change_id}/proposal.md

## Quality Criteria

1. **Summary** is clear and specific (not vague)
2. **Why** section has compelling business/technical value
3. **affected_specs** list is complete and well-scoped
4. **Impact** analysis covers all affected areas
5. **Formatting** follows markdown standards

## Instructions

1. **Read the proposal file**:
   ```bash
   agentd file read {change_id} proposal
   ```

2. Check against quality criteria

If issues found:
  1. Create updated proposal JSON
  2. Write to /tmp/proposal-{change_id}.json using heredoc
  3. Run: agentd proposal create {change_id} --json-file /tmp/proposal-{change_id}.json
  4. Output: `<review>NEEDS_REVISION</review>`

If no issues:
  1. Output: `<review>PASS</review>`

IMPORTANT: You MUST output exactly one of the two markers above.
"#,
        change_id = change_id
    )
}

/// Generate self-review prompt for a spec (sequential generation)
pub fn spec_self_review_with_mcp_prompt(change_id: &str, spec_id: &str, other_specs: &[String]) -> String {
    let other_specs_list = if other_specs.is_empty() {
        String::new()
    } else {
        format!(
            "\nOther specs (check consistency):\n{}",
            other_specs.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
        )
    };

    format!(
        r#"## Task: Review spec '{spec_id}'

Read and review: agentd/changes/{change_id}/specs/{spec_id}.md

## Context Files:
- agentd/changes/{change_id}/proposal.md (for reference){other_specs_list}

## Quality Criteria

1. **Requirements** are testable and clear
2. **Scenarios** cover happy path, errors, edge cases (min 3)
3. **Consistent** with proposal.md
4. **Consistent** with other specs (no duplicate definitions)
5. **Mermaid diagrams** are correct (if present)
6. **Formatting** follows markdown standards

## Instructions

1. **Read the spec file and context**:
   ```bash
   agentd file read {change_id} {spec_id}
   agentd file read {change_id} proposal
   agentd spec list {change_id}
   ```

2. Check against quality criteria

If issues found:
  1. Create updated spec JSON
  2. Write to /tmp/spec-{change_id}-{spec_id}.json using heredoc
  3. Run: agentd spec create {change_id} {spec_id} --json-file /tmp/spec-{change_id}-{spec_id}.json
  4. Output: `<review>NEEDS_REVISION</review>`

If no issues:
  1. Output: `<review>PASS</review>`

IMPORTANT: You MUST output exactly one of the two markers above.
"#,
        change_id = change_id,
        spec_id = spec_id,
        other_specs_list = other_specs_list
    )
}

/// Generate self-review prompt for tasks.md (sequential generation)
pub fn tasks_self_review_with_mcp_prompt(change_id: &str, all_files: &[String]) -> String {
    let context_list = all_files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n");

    format!(
        r#"## Task: Review tasks.md

Read and review: agentd/changes/{change_id}/tasks.md

## Context Files:
{context_list}

## Quality Criteria

1. **Coverage**: All spec requirements are covered by tasks
2. **Dependencies**: Correct, no circular deps
3. **Layer organization**: Logical (data → logic → integration → testing)
4. **File paths**: Accurate and specific
5. **Spec refs**: Each task has clear spec_ref
6. **Formatting**: Follows markdown standards

## Instructions

1. **Read tasks.md and all context files**:
   ```bash
   agentd file read {change_id} tasks
   agentd file read {change_id} proposal
   agentd spec list {change_id}
   ```

2. Check against quality criteria

If issues found:
  1. Create updated tasks JSON
  2. Write to /tmp/tasks-{change_id}.json using heredoc
  3. Run: agentd tasks create {change_id} --json-file /tmp/tasks-{change_id}.json
  4. Output: `<review>NEEDS_REVISION</review>`

If no issues:
  1. Output: `<review>PASS</review>`

IMPORTANT: You MUST output exactly one of the two markers above.
"#,
        change_id = change_id,
        context_list = context_list
    )
}

/// Generate Gemini reproposal prompt (for resuming sessions)
pub fn gemini_reproposal_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Instructions

**IMPORTANT**: You MUST use CLI workflow to update files. Do NOT edit files directly.

1. **Read the review feedback**:
   ```bash
   agentd file read {change_id} proposal
   ```
   Look for <review> blocks with issues to address

2. **Address each issue** using the appropriate CLI workflow:
   - For proposal.md: Create JSON → Write to /tmp/proposal-{change_id}.json → Run: agentd proposal create {change_id} --json-file /tmp/proposal-{change_id}.json
   - For spec files: Create JSON → Write to /tmp/spec-{change_id}-SPEC_ID.json → Run: agentd spec create {change_id} SPEC_ID --json-file /tmp/spec-{change_id}-SPEC_ID.json
   - For tasks.md: Create JSON → Write to /tmp/tasks-{change_id}.json → Run: agentd tasks create {change_id} --json-file /tmp/tasks-{change_id}.json

Direct file editing is NOT allowed. Use CLI workflow only.
"#,
        change_id = change_id
    )
}

/// Generate Gemini self-review prompt for reviewing all proposal files
pub fn proposal_self_review_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Self-Review Task
Review all generated proposal files in agentd/changes/{change_id}/:
- proposal.md (PRD)
- tasks.md
- specs/*.md

## Quality Criteria
1. **Completeness**: All required frontmatter fields present and valid
2. **Consistency**: IDs, references, and versions match across files
3. **Clarity**: Requirements and tasks are specific and actionable
4. **Structure**: Proper markdown formatting, valid YAML blocks
5. **Traceability**: Tasks reference specs, specs have clear requirements

## Instructions

**IMPORTANT**: You MUST use CLI workflow to fix any issues. Do NOT edit files directly.

1. **Read all files**:
   ```bash
   agentd implementation read-all {change_id}
   ```

2. Check against the quality criteria

3. If ANY issues are found:
   - Use CLI workflow to fix them:
     - For proposal.md: Create JSON → Write to /tmp/proposal-{change_id}.json → Run: agentd proposal create {change_id} --json-file /tmp/proposal-{change_id}.json
     - For spec files: Create JSON → Write to /tmp/spec-{change_id}-SPEC_ID.json → Run: agentd spec create {change_id} SPEC_ID --json-file /tmp/spec-{change_id}-SPEC_ID.json
     - For tasks.md: Create JSON → Write to /tmp/tasks-{change_id}.json → Run: agentd tasks create {change_id} --json-file /tmp/tasks-{change_id}.json
   - Output: `<review>NEEDS_REVISION</review>`

4. If NO issues are found:
   - Output: `<review>PASS</review>`

CRITICAL: Direct file editing is NOT allowed. Use CLI workflow only.
IMPORTANT: You MUST output exactly one of the two markers above at the end of your response.
"#,
        change_id = change_id
    )
}

/// Generate Gemini spec merge prompt
pub fn gemini_merge_specs_prompt(change_id: &str, strategy: &str, spec_file: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Merge Strategy
{strategy}

## Spec File
{spec_file}

## Instructions
Merge delta specs from agentd/changes/{change_id}/specs/ back to main specs/.
Apply the specified merge strategy.
"#,
        change_id = change_id,
        strategy = strategy,
        spec_file = spec_file
    )
}

/// Generate Gemini changelog prompt
pub fn gemini_changelog_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Instructions
Generate CHANGELOG.md entry for agentd/changes/{change_id}/.
Review implemented changes and create a user-facing changelog.
"#,
        change_id = change_id
    )
}

/// Generate Gemini fillback prompt
pub fn gemini_fillback_prompt(change_id: &str, file_path: &str, placeholder: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## File
{file_path}

## Placeholder
{placeholder}

## Instructions
Fill in the placeholder in the specified file with appropriate content.
"#,
        change_id = change_id,
        file_path = file_path,
        placeholder = placeholder
    )
}

/// Generate Gemini archive fix prompt
pub fn gemini_archive_fix_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Instructions
Fix issues found in the archive quality review.
Read agentd/changes/{change_id}/ARCHIVE_REVIEW.md for issues to fix.
"#,
        change_id = change_id
    )
}

/// Generate Codex challenge prompt
pub fn codex_challenge_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Instructions

1. **Get Requirements**:
   ```bash
   agentd implementation read-all {change_id}
   ```
   This will retrieve proposal.md, tasks.md, and all specs/*.md in one call

2. **Review for Content/Logical Issues**:
   - **Completeness** - Are all requirements covered? Missing scenarios?
   - **Consistency** - Do specs align with proposal? Do tasks cover all requirements?
   - **Technical feasibility** - Is the design implementable? Any blockers?
   - **Clarity** - Are requirements specific and testable? Ambiguous language?
   - **Dependencies** - Are task dependencies correct? Missing prerequisites?

3. **Submit Review**: Use CLI workflow with your findings

**IMPORTANT**:
- DO NOT check format issues - CLI tools guarantee correct structure
- Focus ONLY on content/logical issues
- You MUST use CLI workflow - direct file access is NOT allowed

## Review Submission

1. **Create review JSON** with this structure:
   ```json
   {{
     "status": "approved|needs_revision|rejected",
     "iteration": 1,
     "reviewer": "codex",
     "content": "Markdown string with:\n## Summary\n## Issues\n## Verdict\n## Next Steps"
   }}
   ```

2. **Write JSON to file** using heredoc:
   ```bash
   cat > /tmp/review-{change_id}.json <<'EOF'
   {{ JSON content here }}
   EOF
   ```

3. **Run CLI command**:
   ```bash
   agentd proposal review {change_id} --json-file /tmp/review-{change_id}.json
   ```

Your review content MUST include these sections:
- ## Summary - Overall assessment
- ## Issues - List of CONTENT issues found (if any)
- ## Verdict - APPROVED, NEEDS_REVISION, or REJECTED
- ## Next Steps - Recommendations

## Verdict Guidelines
- **approved**: Content is complete, consistent, and ready for implementation
- **needs_revision**: Has logical issues (missing requirements, inconsistencies, unclear specs)
- **rejected**: Fundamental design problems that require starting over
"#,
        change_id = change_id
    )
}

/// Generate Codex rechallenge prompt (for resuming sessions)
pub fn codex_rechallenge_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Instructions

1. **Get Requirements**:
   ```bash
   agentd implementation read-all {change_id}
   ```
   This retrieves the updated proposal, tasks, and specs

2. **Review Updates**: Focus ONLY on whether previous content issues have been addressed
   - DO NOT check format issues - CLI tools guarantee correct structure
   - Check if logical issues from previous review are resolved

3. **Submit Follow-up Review**: Use CLI workflow with your findings
   - Create review JSON (increment iteration number)
   - Write to /tmp/review-{change_id}.json using heredoc
   - Run: agentd proposal review {change_id} --json-file /tmp/review-{change_id}.json

**IMPORTANT**: You MUST use CLI workflow - direct file access is NOT allowed.
"#,
        change_id = change_id
    )
}

/// Generate Codex review prompt with pre-processing results
pub fn codex_review_prompt(
    change_id: &str,
    iteration: u32,
    test_output: &str,
    audit_output: &str,
    semgrep_output: &str,
    clippy_output: &str,
) -> String {
    format!(
        r#"# Agentd Code Review Task (Iteration {iteration})

Change ID: {change_id}

## Test Results (Embedded)
```
{test_output}
```

## Security Audit Results (Embedded)
```
{audit_output}
```

## Semgrep Results (Embedded)
```
{semgrep_output}
```

## Clippy Results (Embedded)
```
{clippy_output}
```

## Instructions

1. **Get Requirements**:
   ```bash
   agentd implementation read-all {change_id}
   ```
   This retrieves proposal.md, tasks.md, and all specs/*.md

2. **Get Implementation Summary**:
   ```bash
   agentd implementation list-files {change_id}
   ```
   This provides git diff summary, changed files, and commit log
   - For detailed code review, use the `Read` tool on specific files

3. **Analyze Test Results** (embedded above):
   - Parse test pass/fail status
   - Identify failing tests and reasons
   - Calculate coverage if available

4. **Analyze Security Scan Results** (embedded above):
   - Parse cargo audit for dependency vulnerabilities
   - Parse semgrep for security patterns
   - Parse clippy for code quality and security warnings

5. **Review Code Quality**:
   - Best practices, performance, error handling
   - Requirement compliance (match proposal/specs)
   - Consistency with existing patterns

6. **Write Review**: Create agentd/changes/{change_id}/REVIEW.md with comprehensive findings

## Review Focus
1. **Test Results (HIGH)**: Are all tests passing? Coverage adequate?
2. **Security (HIGH)**: Any vulnerabilities from tools? Security best practices?
3. **Best Practices (HIGH)**: Performance, error handling, style
4. **Requirement Compliance (HIGH)**: Does code match proposal/specs?
5. **Consistency (MEDIUM)**: Does code follow existing patterns?
6. **Test Quality (MEDIUM)**: Are tests comprehensive and well-written?

## Severity Guidelines
- **HIGH**: Failing tests, security vulnerabilities, missing features, wrong behavior
- **MEDIUM**: Style inconsistencies, missing tests, minor improvements
- **LOW**: Suggestions, nice-to-haves

## Verdict Guidelines
- **APPROVED**: All tests pass, no HIGH issues (LOW/MEDIUM issues acceptable)
- **NEEDS_CHANGES**: Some tests fail or HIGH/MEDIUM issues exist (fixable)
- **MAJOR_ISSUES**: Many failing tests or critical security issues

Be thorough but fair. Include iteration number in REVIEW.md.
"#,
        change_id = change_id,
        iteration = iteration,
        test_output = test_output,
        audit_output = audit_output,
        semgrep_output = semgrep_output,
        clippy_output = clippy_output
    )
}

/// Generate Codex verify prompt
pub fn codex_verify_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Instructions

1. **Get Requirements**:
   ```bash
   agentd implementation read-all {change_id}
   ```
   This retrieves proposal.md, tasks.md, and all specs/*.md

2. **Verify Implementation**: Ensure all tasks and requirements are complete
   - Check each requirement is satisfied
   - Verify all tasks are implemented
   - Use: agentd implementation list-files {change_id} to see changed files

3. **Write Verification Results**: Create VERIFY.md with findings
"#,
        change_id = change_id
    )
}

/// Generate Codex archive review prompt
pub fn codex_archive_review_prompt(change_id: &str, strategy: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Archive Strategy
{strategy}

## Instructions

1. **Get Requirements**:
   ```bash
   agentd implementation read-all {change_id}
   ```
   This retrieves proposal.md, tasks.md, and all specs/*.md

2. **Review Archive Quality**:
   - Check completeness of documentation
   - Verify changelog is comprehensive
   - Ensure all artifacts are properly archived

3. **Write Archive Review**: Create ARCHIVE_REVIEW.md with findings
"#,
        change_id = change_id,
        strategy = strategy
    )
}

/// Generate Claude implementation prompt
pub fn claude_implement_prompt(change_id: &str, tasks: Option<&str>) -> String {
    let task_filter = if let Some(t) = tasks {
        format!("Only implement tasks: {}", t)
    } else {
        String::new()
    };

    format!(
        r#"# Agentd Implement Task

Implement the proposal for agentd/changes/{change_id}/.

## Instructions
1. Read proposal.md, tasks.md, and specs/
2. Implement ALL tasks in tasks.md {task_filter}
3. **Write tests for all implemented features** (unit + integration)
   - Test all spec scenarios (WHEN/THEN cases)
   - Include edge cases and error handling
   - Use existing test framework patterns
4. Create/update IMPLEMENTATION.md with progress notes

## Code Quality
- Follow existing code style and patterns
- Add proper error handling
- Include documentation comments where needed

**IMPORTANT**: Write comprehensive tests. Tests are as important as the code itself.
"#,
        change_id = change_id,
        task_filter = task_filter
    )
}

/// Generate Claude resolve prompt
pub fn claude_resolve_prompt(change_id: &str) -> String {
    format!(
        r#"# Agentd Resolve Task

Fix issues found during code review for agentd/changes/{change_id}/.

## Instructions
1. Read REVIEW.md to understand the issues
2. Fix all HIGH severity issues
3. Fix MEDIUM severity issues if feasible
4. Update IMPLEMENTATION.md with fix notes
5. Ensure all tests pass after fixes

## Code Quality
- Follow existing code style and patterns
- Add proper error handling
- Maintain or improve test coverage
"#,
        change_id = change_id
    )
}

/// Generate Claude spec-level implementation prompt
pub fn claude_implement_spec_prompt(change_id: &str, spec_id: &str) -> String {
    format!(
        r#"# Agentd Implement Spec: {spec_id}

Implement the tasks for spec `{spec_id}` in agentd/changes/{change_id}/.

## Spec to Implement
Read: agentd/changes/{change_id}/specs/{spec_id}.md

## Tasks for This Spec
Find all tasks in agentd/changes/{change_id}/tasks.md that reference spec `{spec_id}`.

## Instructions
1. Read the spec file to understand requirements
2. Read tasks.md to find all tasks for this spec
3. Implement ONLY the tasks for this spec
4. Write tests for implemented features (unit + integration)
   - Test all spec scenarios (WHEN/THEN cases)
   - Include edge cases and error handling
5. Update IMPLEMENTATION.md with notes for this spec

## Focus
- Implement ONLY this spec's requirements
- Other specs will be handled separately
- Ensure this spec's acceptance criteria are met

**IMPORTANT**: Write comprehensive tests for this spec's scenarios.
"#,
        change_id = change_id,
        spec_id = spec_id
    )
}

/// Generate Claude spec-level self-review prompt
pub fn claude_self_review_spec_prompt(change_id: &str, spec_id: &str) -> String {
    format!(
        r#"# Self-Review: Spec {spec_id}

Review your implementation of spec `{spec_id}` in agentd/changes/{change_id}/.

## What to Check
1. Read the spec: agentd/changes/{change_id}/specs/{spec_id}.md
2. Verify all tasks for this spec are implemented correctly
3. Check that tests cover all scenarios (WHEN/THEN cases)
4. Verify code follows existing patterns
5. Check for any obvious bugs or issues

## Output Format
Provide a brief review in this format:

```yaml
spec: {spec_id}
status: PASS | NEEDS_FIX
issues:
  - [Issue description if NEEDS_FIX]
```

Be critical but fair. If implementation looks good, output a message containing "✅" or "PASS". If there are issues, list them clearly and the message should indicate problems found.
"#,
        change_id = change_id,
        spec_id = spec_id
    )
}

/// Generate Claude spec-level fix prompt
pub fn claude_resolve_spec_prompt(change_id: &str, spec_id: &str) -> String {
    format!(
        r#"# Fix Issues: Spec {spec_id}

Fix the issues found during self-review of spec `{spec_id}` in agentd/changes/{change_id}/.

## Instructions
1. Review the self-review feedback from the previous response
2. Fix all identified issues
3. Ensure tests pass
4. Update IMPLEMENTATION.md with fix notes

## Code Quality
- Follow existing code style and patterns
- Add proper error handling
- Maintain test coverage
"#,
        change_id = change_id,
        spec_id = spec_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_proposal_prompt() {
        let prompt = gemini_proposal_prompt("test-change", "Add new feature");
        assert!(prompt.contains("test-change"));
        assert!(prompt.contains("Add new feature"));
        assert!(prompt.contains("agentd proposal create"));
        assert!(prompt.contains("CLI workflow"));
    }

    #[test]
    fn test_codex_review_prompt() {
        let prompt = codex_review_prompt(
            "test-change",
            1,
            "tests passed",
            "no vulnerabilities",
            "no issues",
            "no warnings",
        );
        assert!(prompt.contains("test-change"));
        assert!(prompt.contains("Iteration 1"));
        assert!(prompt.contains("tests passed"));
        assert!(prompt.contains("no vulnerabilities"));
    }

    #[test]
    fn test_claude_implement_prompt_with_tasks() {
        let prompt = claude_implement_prompt("test-change", Some("1.1,1.2"));
        assert!(prompt.contains("test-change"));
        assert!(prompt.contains("Only implement tasks: 1.1,1.2"));
    }

    #[test]
    fn test_claude_implement_prompt_without_tasks() {
        let prompt = claude_implement_prompt("test-change", None);
        assert!(prompt.contains("test-change"));
        assert!(!prompt.contains("Only implement tasks"));
    }

    #[test]
    fn test_gemini_merge_specs_prompt() {
        let prompt = gemini_merge_specs_prompt("test-change", "merge", "spec.md");
        assert!(prompt.contains("test-change"));
        assert!(prompt.contains("merge"));
        assert!(prompt.contains("spec.md"));
    }

    // Additional comprehensive tests

    #[test]
    fn test_all_prompts_contain_change_id() {
        let change_id = "my-feature";

        assert!(gemini_proposal_prompt(change_id, "desc").contains(change_id));
        assert!(gemini_reproposal_prompt(change_id).contains(change_id));
        assert!(proposal_self_review_prompt(change_id).contains(change_id));
        assert!(gemini_merge_specs_prompt(change_id, "strategy", "file").contains(change_id));
        assert!(gemini_changelog_prompt(change_id).contains(change_id));
        assert!(gemini_fillback_prompt(change_id, "path", "placeholder").contains(change_id));
        assert!(gemini_archive_fix_prompt(change_id).contains(change_id));
        assert!(codex_challenge_prompt(change_id).contains(change_id));
        assert!(codex_rechallenge_prompt(change_id).contains(change_id));
        assert!(codex_review_prompt(change_id, 1, "", "", "", "").contains(change_id));
        assert!(codex_verify_prompt(change_id).contains(change_id));
        assert!(codex_archive_review_prompt(change_id, "strategy").contains(change_id));
        assert!(claude_implement_prompt(change_id, None).contains(change_id));
        assert!(claude_resolve_prompt(change_id).contains(change_id));
    }

    #[test]
    fn test_proposal_self_review_prompt() {
        let prompt = proposal_self_review_prompt("test-change");
        assert!(prompt.contains("test-change"));
        assert!(prompt.contains("Self-Review Task"));
        assert!(prompt.contains("<review>PASS</review>"));
        assert!(prompt.contains("<review>NEEDS_REVISION</review>"));
        assert!(prompt.contains("Quality Criteria"));
    }

    #[test]
    fn test_gemini_reproposal_prompt_has_instructions() {
        let prompt = gemini_reproposal_prompt("test");
        assert!(prompt.contains("Instructions"));
        assert!(prompt.contains("CLI workflow"));
        assert!(prompt.contains("agentd proposal create"));
    }

    #[test]
    fn test_gemini_changelog_prompt_format() {
        let prompt = gemini_changelog_prompt("add-auth");
        assert!(prompt.contains("## Change ID"));
        assert!(prompt.contains("add-auth"));
        assert!(prompt.contains("## Instructions"));
        assert!(prompt.contains("CHANGELOG.md"));
    }

    #[test]
    fn test_gemini_fillback_prompt_parameters() {
        let prompt = gemini_fillback_prompt("test", "/path/to/file.rs", "{{placeholder}}");
        assert!(prompt.contains("/path/to/file.rs"));
        assert!(prompt.contains("{{placeholder}}"));
    }

    #[test]
    fn test_codex_challenge_prompt_structure() {
        let prompt = codex_challenge_prompt("new-feature");
        assert!(prompt.contains("## Change ID"));
        assert!(prompt.contains("new-feature"));
        assert!(prompt.contains("## Instructions"));
        assert!(prompt.contains("agentd proposal review"));
        assert!(prompt.contains("CLI workflow"));
    }

    #[test]
    fn test_codex_review_prompt_includes_all_outputs() {
        let test_out = "test output";
        let audit_out = "audit output";
        let semgrep_out = "semgrep output";
        let clippy_out = "clippy output";

        let prompt = codex_review_prompt("test", 2, test_out, audit_out, semgrep_out, clippy_out);

        assert!(prompt.contains(test_out));
        assert!(prompt.contains(audit_out));
        assert!(prompt.contains(semgrep_out));
        assert!(prompt.contains(clippy_out));
        assert!(prompt.contains("Iteration 2"));
    }

    #[test]
    fn test_codex_review_prompt_severity_guidelines() {
        let prompt = codex_review_prompt("test", 1, "", "", "", "");
        assert!(prompt.contains("Severity Guidelines"));
        assert!(prompt.contains("**HIGH**"));
        assert!(prompt.contains("**MEDIUM**"));
        assert!(prompt.contains("**LOW**"));
    }

    #[test]
    fn test_codex_review_prompt_verdict_guidelines() {
        let prompt = codex_review_prompt("test", 1, "", "", "", "");
        assert!(prompt.contains("Verdict Guidelines"));
        assert!(prompt.contains("**APPROVED**"));
        assert!(prompt.contains("**NEEDS_CHANGES**"));
        assert!(prompt.contains("**MAJOR_ISSUES**"));
    }

    #[test]
    fn test_codex_archive_review_prompt_strategy() {
        let prompt = codex_archive_review_prompt("test", "incremental");
        assert!(prompt.contains("incremental"));
        assert!(prompt.contains("Archive Strategy"));
    }

    #[test]
    fn test_claude_implement_prompt_test_requirements() {
        let prompt = claude_implement_prompt("test", None);
        assert!(prompt.contains("**Write tests for all implemented features**"));
        assert!(prompt.contains("unit + integration"));
        assert!(prompt.contains("WHEN/THEN cases"));
    }

    #[test]
    fn test_claude_resolve_prompt_focus() {
        let prompt = claude_resolve_prompt("test");
        assert!(prompt.contains("REVIEW.md"));
        assert!(prompt.contains("Fix all HIGH severity issues"));
        assert!(prompt.contains("IMPLEMENTATION.md"));
    }

    #[test]
    fn test_prompts_with_special_characters() {
        // Test that prompts handle special characters correctly
        let change_id = "feature-with-special-chars-123_test";
        let description = "Add feature with \"quotes\" and 'apostrophes'";

        let prompt = gemini_proposal_prompt(change_id, description);
        assert!(prompt.contains(change_id));
        assert!(prompt.contains(description));
    }

    #[test]
    fn test_empty_iteration_number() {
        let prompt = codex_review_prompt("test", 0, "", "", "", "");
        assert!(prompt.contains("Iteration 0"));
    }

    #[test]
    fn test_high_iteration_number() {
        let prompt = codex_review_prompt("test", 999, "", "", "", "");
        assert!(prompt.contains("Iteration 999"));
    }
}
