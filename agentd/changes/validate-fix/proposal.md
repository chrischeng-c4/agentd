# Change: validate-fix

## Summary
Add a `--fix` flag to the `validate-proposal` command to automatically correct common formatting issues in proposal and spec files.

## Why
Manually fixing repetitive validation errors (such as missing frontmatter, incorrect heading levels, or missing section headers) is tedious and slows down the proposal process. An auto-fixer can handle these deterministic issues, allowing developers to focus on the content.

## What Changes
- Add `--fix` flag to `validate-proposal` command.
- Implement an `AutoFixer` component in the `validator` module.
- `AutoFixer` will support fixing:
    - Missing Frontmatter (injecting skeletons).
    - Missing standard headings (e.g., `## Acceptance Criteria`).
    - Basic Markdown formatting issues (if applicable).
- Update `ValidationOptions` to include the `fix` flag.
- Update `validate_proposal` flow to invoke the fixer when errors are found and the flag is set.

## Impact
- **Affected specs**: None (new feature).
- **Affected code**:
    - `src/main.rs`: Command argument definitions.
    - `src/cli/validate_proposal.rs`: Command argument parsing and flow control.
    - `src/models/validation.rs`: Update `ValidationOptions`.
    - `src/validator/`: Add `fix.rs` and update `mod.rs`.
- **Breaking changes**: No.