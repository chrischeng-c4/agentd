# Tasks

<meta>
  <purpose>Implementation tickets derived from specs</purpose>
  <constraint>NO actual code - just file paths, actions, and references</constraint>
</meta>

<format>
Each task MUST have:
- [ ] {layer}.{number} {Title}
  - File: `exact/path/to/file.ext` (CREATE|MODIFY|DELETE)
  - Spec: `specs/{name}.md#{section}`
  - Do: [WHAT to implement, not HOW]
  - Depends: [task IDs, or "none"]
</format>

<section id="data-layer">
## 1. Data Layer

- [ ] 1.1 [Task title]
  - File: `path/to/file.rs` (CREATE|MODIFY|DELETE)
  - Spec: `specs/[name].md#data-model`
  - Do: [What to implement - not how]
  - Depends: none

<quality>
- Data models, schemas, database migrations
- Should be implemented first (no dependencies)
</quality>
</section>

<section id="logic-layer">
## 2. Logic Layer

- [ ] 2.1 [Task title]
  - File: `path/to/file.rs` (CREATE|MODIFY)
  - Spec: `specs/[name].md#interfaces`
  - Do: [What to implement]
  - Depends: 1.1

<quality>
- Core business logic, handlers, services
- Depends on data layer
</quality>
</section>

<section id="integration">
## 3. Integration

- [ ] 3.1 [Task title]
  - File: `path/to/file.rs` (MODIFY)
  - Spec: `specs/[name].md#flow`
  - Do: [What to integrate]
  - Depends: 2.1

<quality>
- Wire up components, routes, CLI commands
- Depends on logic layer
</quality>
</section>

<section id="testing">
## 4. Testing

- [ ] 4.1 [Test task title]
  - File: `path/to/test.rs` (CREATE)
  - Verify: `specs/[name].md#acceptance-criteria`
  - Depends: 3.1

<quality>
- Unit tests, integration tests
- Reference acceptance criteria from specs
- Should cover all scenarios
</quality>
</section>
