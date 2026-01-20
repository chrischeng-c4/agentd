# tasks.md Format Inconsistency Between Sequential and Non-Sequential Mode

## Problem

The `tasks.md` file format requirements differ between sequential and non-sequential implementation modes:

- **Sequential mode** (`sequential_implementation = true`): Requires YAML frontmatter
- **Non-sequential mode** (`sequential_implementation = false`): Works without frontmatter

This inconsistency forces users to modify their `tasks.md` when switching between modes, which violates the principle of configuration independence.

## Current Behavior

### Sequential Mode (task_graph.rs)
Expects frontmatter with layers definition:
```yaml
---
change_id: test-change
layers:
  logic:
    order: 1
---
```

Parser fails with: `Error: No frontmatter start found`

### Non-Sequential Mode (legacy)
Works with simple markdown format without frontmatter:
```markdown
# Tasks

## 1. Logic Layer
- [ ] 1.1 Task description
  ...
```

## Expected Behavior

Both modes should accept the **same `tasks.md` format**. The recommended approach:

1. **Make frontmatter optional for sequential mode**
   - Default to standard layer ordering if no frontmatter: data(1) → logic(2) → integration(3) → testing(4)
   - Parse layers from markdown headings (e.g., "## 1. Logic Layer")
   
2. **Or make frontmatter mandatory for both modes**
   - Update non-sequential mode to also require frontmatter
   - Ensures consistency but requires migration

## Recommended Solution

**Option 1** (backward compatible):
- Modify `TaskGraph::parse_frontmatter()` to make frontmatter optional
- If no frontmatter found, infer layers from markdown section headings
- Extract layer name and order from headings like "## 2. Logic Layer"

```rust
fn parse_frontmatter(content: &str) -> Result<TasksFrontmatter> {
    if let Some(frontmatter_start) = content.find("---") {
        // Parse YAML frontmatter (current behavior)
        ...
    } else {
        // Infer from markdown headings
        Self::infer_frontmatter_from_markdown(content)
    }
}
```

## Files Affected

- `src/models/task_graph.rs` - Frontmatter parsing logic
- `src/models/frontmatter.rs` - TasksFrontmatter struct (make fields optional)

## Steps to Reproduce

1. Create a `tasks.md` without frontmatter (like `simple-add/tasks.md`)
2. Set `sequential_implementation = true` in config
3. Run `agentd implement <change-id>`
4. Observe: `Error: No frontmatter start found`

## Test Cases

After fix, both formats should work in both modes:

### Format 1: With Frontmatter
```yaml
---
change_id: test
layers:
  logic:
    order: 1
---

## 1. Logic Layer
- [ ] 1.1 Task
```

### Format 2: Without Frontmatter (inferred)
```markdown
# Tasks

## 1. Data Layer
...

## 2. Logic Layer
- [ ] 2.1 Task
```

Both should work with `sequential_implementation = true` or `false`.

## Priority

**Medium** - Affects user experience when switching between implementation modes.

## Labels

- bug
- consistency
- backward-compatibility
