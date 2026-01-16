# Tasks

## 1. Research and Design
- [ ] 1.1 Analyze existing `agentd:proposal` output quality
  - File: `.gemini/commands/agentd/proposal.toml` (MODIFY)
  - Spec: `specs/proposal-prompt.md#prompt-architecture`
  - Do: Identify specific areas where the current prompt fails to provide enough guidance.

## 2. Implementation
- [ ] 2.1 Update `proposal.toml` with engineered prompt
  - File: `.gemini/commands/agentd/proposal.toml` (MODIFY)
  - Spec: `specs/proposal-prompt.md#prompt-architecture`
  - Do: Replace the `prompt` string with the improved version incorporating persona, CoT, and better constraints.

## 3. Verification
- [ ] 3.1 Test the new prompt with a sample request
  - File: `agentd/changes/test-proposal/` (CREATE)
  - Verify: `specs/proposal-prompt.md#acceptance-criteria`
  - Do: Run `agentd proposal` and verify the quality of the generated files.