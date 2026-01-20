# Challenge Report: mcp-spec-tool-2

## Summary
The proposal direction is feasible, but multiple spec requirements are not covered by tasks, and the CHALLENGE.md removal is incomplete across the codebase. These gaps will block implementation unless addressed.

## Internal Consistency Issues

### Issue: CHALLENGE.md removal scope is incomplete
- **Severity**: High
- **Category**: Completeness
- **Description**: Proposal and spec require removing CHALLENGE.md entirely, but tasks only cover a subset of references. Numerous code paths and templates still depend on CHALLENGE.md and are not listed in tasks, which will leave dangling references and failing tests.
- **Location**: `templates/codex/prompts/agentd-challenge.md`, `templates/skills/agentd-plan/SKILL.md`, `src/cli/init.rs`, `src/models/frontmatter.rs`, `src/models/challenge.rs`, `src/models/annotation.rs`, `src/parser/inline_yaml.rs`, `src/ui/viewer/assets/app.js`, `src/ui/viewer/render.rs`
- **Recommendation**: Expand tasks to cover all CHALLENGE.md references (templates, models, parser, UI assets/tests, init tooling) or explicitly scope what remains and why.

### Issue: Prompt schema/example requirement not reflected in tasks
- **Severity**: High
- **Category**: Completeness
- **Description**: R1 mandates that prompts include JSON schemas and examples for MCP tool calls. Tasks only say “require MCP tools” without adding schemas/examples, so the requirement is not implemented.
- **Location**: `agentd/changes/mcp-spec-tool-2/specs/mcp-tool-enforcement.md#R1`, `agentd/changes/mcp-spec-tool-2/tasks.md` (2.1–2.3)
- **Recommendation**: Add explicit tasks to embed tool schemas and example calls in all relevant prompts/templates.

### Issue: Metadata preservation and validation apply to all MCP tools, but tasks target only a subset
- **Severity**: High
- **Category**: Completeness
- **Description**: R2 and R4 apply to MCP tools broadly, but tasks only update `create_proposal` and validate in proposal/spec tools. Other MCP tools (e.g., `create_tasks`, `create_clarifications`) remain unaddressed.
- **Location**: `agentd/changes/mcp-spec-tool-2/specs/mcp-tool-enforcement.md#R2`, `agentd/changes/mcp-spec-tool-2/specs/mcp-tool-enforcement.md#R4`, `agentd/changes/mcp-spec-tool-2/tasks.md` (1.3–1.4)
- **Recommendation**: Add tasks to make all MCP tools update-aware and validate inputs according to spec.

### Issue: Self-review bypass detection is required but has no implementation task
- **Severity**: High
- **Category**: Completeness
- **Description**: Acceptance criteria require the self-review phase to detect MCP tool bypass and return NEEDS_REVISION. There is no task covering self-review logic, checks, or prompt changes to enforce this.
- **Location**: `agentd/changes/mcp-spec-tool-2/specs/mcp-tool-enforcement.md#Acceptance Criteria`, `agentd/changes/mcp-spec-tool-2/tasks.md`
- **Recommendation**: Add tasks to implement bypass detection (prompt updates plus any validator or review logic changes required).

## Code Alignment Issues

### Issue: Existing challenge tooling templates conflict with proposed tool-only workflow
- **Severity**: Medium
- **Category**: Conflict
- **Description**: The Codex challenge prompt template still instructs generating CHALLENGE.md directly, which conflicts with the new append_review tool-only flow.
- **Location**: `templates/codex/prompts/agentd-challenge.md`, `src/cli/init.rs`
- **Note**: No refactor/breaking exception is called out here.
- **Recommendation**: Update the template and init flow to reference `append_review` and proposal.md review blocks only.

## Quality Suggestions

### Issue: Migration behavior should be specified for archived changes
- **Severity**: Low
- **Category**: Completeness
- **Description**: The proposal mentions handling existing CHALLENGE.md, but does not clarify expectations for archived changes or viewer behavior when only proposal.md reviews exist.
- **Recommendation**: Add a brief note in specs or tasks on how archived changes and UI viewer should behave after consolidation.

## Verdict
- [ ] APPROVED - Ready for implementation
- [x] NEEDS_REVISION - Address issues above (spec/task coverage gaps)
- [ ] REJECTED - Fundamental problems, needs rethinking

**Next Steps**: Expand tasks/specs to cover missing requirements and update all remaining CHALLENGE.md references before implementation.
