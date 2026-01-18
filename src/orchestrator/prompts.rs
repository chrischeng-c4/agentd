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
Create proposal files in agentd/changes/{change_id}/.
"#,
        change_id = change_id,
        description = description
    )
}

/// Generate Gemini reproposal prompt (for resuming sessions)
pub fn gemini_reproposal_prompt(change_id: &str) -> String {
    format!(
        r#"## Change ID
{change_id}

## Instructions
Continue refining the proposal based on previous feedback.
Update files in agentd/changes/{change_id}/ as needed.
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
1. Read each file and check against the quality criteria
2. If ANY issues are found:
   - Edit the files directly to fix them
   - Output: `<review>NEEDS_REVISION</review>`
3. If NO issues are found:
   - Output: `<review>PASS</review>`

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
Review the proposal in agentd/changes/{change_id}/.
Analyze technical feasibility, potential issues, and suggest improvements.
Create CHALLENGE.md with findings.
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
Continue challenging the proposal based on previous analysis.
Update CHALLENGE.md in agentd/changes/{change_id}/ as needed.
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

Review the implementation for agentd/changes/{change_id}/.

## Test Results
```
{test_output}
```

## Security Audit Results
```
{audit_output}
```

## Semgrep Results
```
{semgrep_output}
```

## Clippy Results
```
{clippy_output}
```

## Instructions
1. Read proposal.md, tasks.md, specs/ to understand requirements
2. Read implemented code (search for new/modified files)
3. **Analyze test results**:
   - Parse test pass/fail status
   - Identify failing tests and reasons
   - Calculate coverage if available
4. **Analyze security scan results**:
   - Parse cargo audit for dependency vulnerabilities
   - Parse semgrep for security patterns
   - Parse clippy for code quality and security warnings
5. Review code quality, best practices, and requirement compliance
6. Fill agentd/changes/{change_id}/REVIEW.md with comprehensive findings

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
Verify the implementation against requirements.
Read agentd/changes/{change_id}/ and ensure all tasks are complete.
Create VERIFY.md with verification results.
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
Review the archive quality before finalizing.
Check completeness, documentation, and changelog.
Create ARCHIVE_REVIEW.md with findings.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_proposal_prompt() {
        let prompt = gemini_proposal_prompt("test-change", "Add new feature");
        assert!(prompt.contains("test-change"));
        assert!(prompt.contains("Add new feature"));
        assert!(prompt.contains("Create proposal files"));
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
        assert!(prompt.contains("Continue refining"));
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
        assert!(prompt.contains("CHALLENGE.md"));
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
