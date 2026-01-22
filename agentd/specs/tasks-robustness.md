# Specification: Tasks Frontmatter & Inferred Layers

## Overview

Improve the robustness of `tasks.md` parsing by making frontmatter optional and supporting both `id` and `change_id` fields. If frontmatter is missing, the system will infer layer definitions from markdown headings to maintain the task dependency graph.

## Requirements

### TR-R1: Support `change_id` Alias
The `TasksFrontmatter` struct MUST accept both `id` and `change_id` fields in its YAML frontmatter, mapping them to the same internal identifier.

### TR-R2: Resolve Hardcoded Spec Paths
The `TaskGraph` MUST NOT use hardcoded `{{change_id}}` strings in its specification path resolution. It must use the actual `change_id` derived from the environment or frontmatter.

### TR-R3: Inferred Layers from Headings
When frontmatter is missing or incomplete, the system MUST be able to infer layer names and their relative order by scanning markdown headings that follow the `{Order}. {Name}` pattern (e.g., `## 1. Data Layer`).

### TR-R4: Mandatory Frontmatter in Generation
The `tasks_service.rs` MUST include both `id` and `change_id` in the generated YAML frontmatter to ensure maximum compatibility with different versions of the tool.

## Data Model

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["id"],
  "properties": {
    "id": { "type": "string", "description": "The change ID" },
    "change_id": { "type": "string", "description": "Alias for id" },
    "type": { "type": "string", "enum": ["tasks"] },
    "version": { "type": "integer" },
    "layers": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "order"],
        "properties": {
          "name": { "type": "string" },
          "order": { "type": "integer" }
        }
      }
    }
  }
}
```

## Interfaces

```
FUNCTION infer_frontmatter_from_markdown(content: String) -> TasksFrontmatter
  INPUT: Full content of tasks.md markdown
  OUTPUT: A TasksFrontmatter object with layers inferred from "## {n}. {Title}" patterns
  ERRORS: If no layers can be inferred and no frontmatter is present

FUNCTION resolve_spec_path(change_id: String, spec_name: String) -> PathBuf
  INPUT: Change ID and relative spec path
  OUTPUT: Absolute path to the spec file
  SIDE_EFFECTS: Replaces any internal placeholders with actual change_id
```

## Acceptance Criteria

### Scenario: Parsing with change_id in Frontmatter
- **WHEN** `tasks.md` has `change_id: fix-workflow-bugs` in frontmatter
- **THEN** the system correctly identifies the change ID.

### Scenario: Parsing without Frontmatter
- **WHEN** `tasks.md` has no `---` delimiters but contains headings like `## 1. Data Layer` and `## 2. Logic Layer`
- **THEN** the system successfully builds a TaskGraph with two layers in the correct order.

### Scenario: Tasks Generation
- **WHEN** the `plan` workflow generates a new `tasks.md`
- **THEN** the frontmatter includes both `id: fix-workflow-bugs` and `change_id: fix-workflow-bugs`.