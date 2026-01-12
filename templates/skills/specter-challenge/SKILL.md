---
name: specter-challenge
description: Challenge proposal with Codex code analysis
user-invocable: true
---

# /specter:challenge - Challenge Proposal

Analyze proposal against existing codebase using Codex to identify potential issues, conflicts, and improvements.

## Usage

```
/specter:challenge <change-id>
```

**Example**:
```
/specter:challenge add-oauth
```

## What This Skill Does

1. **Load Proposal** - Read proposal, tasks, and spec deltas
2. **Analyze Code** - Use Codex to compare proposal with existing codebase
3. **Identify Issues** - Find conflicts, inconsistencies, and improvements
4. **Generate Report** - Create `CHALLENGE.md` with detailed findings
5. **Display Summary** - Show issue count and severity

## Steps

### 1. Validate Change Exists

```bash
CHANGE_DIR="changes/$CHANGE_ID"
if [ ! -d "$CHANGE_DIR" ]; then
    echo "❌ Error: Change '$CHANGE_ID' not found"
    echo "Run: /specter:proposal $CHANGE_ID \"description\""
    exit 1
fi
```

### 2. Read Proposal Files

```bash
PROPOSAL="$CHANGE_DIR/proposal.md"
TASKS="$CHANGE_DIR/tasks.md"
SPECS="$CHANGE_DIR/specs"

if [ ! -f "$CHANGE_DIR/proposal.md" ]; then
    echo "❌ Error: proposal.md not found"
    exit 1
fi
```

### 3. Call Codex for Challenge Analysis

**Option A: Using codex-cli**:
```bash
codex challenge \
    --proposal "$CHANGE_DIR/proposal.md" \
    --tasks "$CHANGE_DIR/tasks.md" \
    --specs "$CHANGE_DIR/specs" \
    --existing-code "src/" \
    --output "$CHANGE_DIR/CHALLENGE.md"
```

**Option B: Use Claude (you!) to analyze**:

As Claude, you should:
1. Read the proposal files
2. Analyze against existing codebase
3. Identify issues:
   - Architecture conflicts
   - Naming inconsistencies
   - Missing migration paths
   - Potential bugs
4. Generate CHALLENGE.md with issues and suggestions

## Recommendation

Since you're using Claude Code interactively, I recommend:

**修改 `specter init` 來生成 Skills**，然後你就可以在 Claude Code 中使用：
- `/specter:proposal`
- `/specter:challenge`
- `/specter:reproposal`
- 等等

讓我修改 `cli/init.rs` 來實現這個功能。要繼續嗎？