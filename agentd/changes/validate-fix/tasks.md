# Tasks

## 1. CLI Interface
- [ ] 1.1 Add `--fix` flag to `validate-proposal` command
    - File: `src/main.rs` (MODIFY)
    - Spec: `specs/fixer.md#r3-cli-integration`
    - Do: Add `fix: bool` field to `ValidateProposal` variant in `Commands` enum.
- [ ] 1.2 Pass `fix` flag to `ValidationOptions`
    - File: `src/main.rs` (MODIFY)
    - Spec: `specs/fixer.md#r3-cli-integration`
    - Do: Update command handler to pass `fix` flag when creating `ValidationOptions`.

## 2. Models
- [ ] 2.1 Update `ValidationOptions` struct
    - File: `src/models/validation.rs` (MODIFY)
    - Spec: `specs/fixer.md#data-model`
    - Do: Add `pub fix: bool` field and update `new()`/builder methods.

## 3. Validator Logic
- [ ] 3.1 Create `AutoFixer` struct
    - File: `src/validator/fix.rs` (CREATE)
    - Spec: `specs/fixer.md#interfaces`
    - Do: Define `AutoFixer` struct and `new` method.
- [ ] 3.2 Implement `fix_errors` entry point
    - File: `src/validator/fix.rs` (MODIFY)
    - Spec: `specs/fixer.md#flow`
    - Do: Implement the main loop that iterates errors and applies strategies.
- [ ] 3.3 Implement Frontmatter strategy
    - File: `src/validator/fix.rs` (MODIFY)
    - Spec: `specs/fixer.md#r1-frontmatter-injection`
    - Do: Fix `MissingFrontmatter` errors by injecting default frontmatter.
- [ ] 3.4 Implement Heading strategy
    - File: `src/validator/fix.rs` (MODIFY)
    - Spec: `specs/fixer.md#r2-heading-injection`
    - Do: Fix `MissingHeading` errors by appending required headings.
- [ ] 3.5 Register `AutoFixer` module
    - File: `src/validator/mod.rs` (MODIFY)
    - Spec: `specs/fixer.md#interfaces`
    - Do: Add `pub mod fix;` and re-export `AutoFixer`.

## 4. Integration
- [ ] 4.1 Integrate Fixer into Validation Flow
    - File: `src/cli/validate_proposal.rs` (MODIFY)
    - Spec: `specs/fixer.md#flow`
    - Do: In `validate_proposal` function, if `options.fix` is true and errors exist, invoke `AutoFixer`, then re-validate.
