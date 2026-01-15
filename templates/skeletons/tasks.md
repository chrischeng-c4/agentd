# Tasks

<!--
Each task is a dev ticket derived from specs.
NO actual code - just file paths, actions, and references.
-->

## 1. Data Layer
- [ ] 1.1 [Task title]
  - File: `path/to/file.rs` (CREATE|MODIFY|DELETE)
  - Spec: `specs/[name].md#[section]`
  - Do: [What to implement - not how]

## 2. Logic Layer
- [ ] 2.1 [Task title]
  - File: `path/to/file.rs` (CREATE|MODIFY)
  - Spec: `specs/[name].md#[section]`
  - Do: [What to implement]
  - Depends: 1.1

## 3. Integration
- [ ] 3.1 [Task title]
  - File: `path/to/file.rs` (MODIFY)
  - Do: [What to integrate]
  - Depends: 2.1

## 4. Testing
- [ ] 4.1 [Test task title]
  - File: `path/to/test.rs` (CREATE)
  - Verify: `specs/[name].md#acceptance-criteria`
  - Depends: [relevant tasks]
