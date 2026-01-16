#!/bin/bash
# Codex archive review script - reviews merged specs and CHANGELOG for quality
# Usage: ./codex-archive-review.sh <change-id> <strategy>

set -euo pipefail

CHANGE_ID="$1"
STRATEGY="$2"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Reviewing archive quality with Codex: $CHANGE_ID"

# Create ARCHIVE_REVIEW.md skeleton
REVIEW_PATH="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/ARCHIVE_REVIEW.md"
cat > "$REVIEW_PATH" << 'SKELETON'
# Archive Quality Review

## Status: [ ] APPROVED | [ ] NEEDS_FIX | [ ] REJECTED

## Merged Specs Review

[Codex will fill this]

## CHANGELOG Review

[Codex will fill this]

## Overall Assessment

[Summary]

## Issues Found

[List if any]

## Recommendation

- [ ] APPROVED - Safe to archive
- [ ] NEEDS_FIX - Fix issues above
- [ ] REJECTED - Major issues
SKELETON

PROMPT=$(cat << EOF
# Agentd Archive: Quality Review Task

Review the merged specs and CHANGELOG before archiving change: ${CHANGE_ID}

## Context

You need to verify that Gemini correctly merged the spec deltas:
- **Delta specs**: agentd/changes/${CHANGE_ID}/specs/ (original changes)
- **Merged specs**: agentd/specs/ (after Gemini merge)
- **Strategy used**: ${STRATEGY}
- **CHANGELOG**: agentd/specs/CHANGELOG.md (latest entry should be for ${CHANGE_ID})

## Your Task

### 1. Compare Delta vs Merged Specs

**IMPORTANT**: Skip template files (files starting with underscore like _skeleton.md).

For each spec file in the delta (excluding templates):
1. Read the delta spec: agentd/changes/${CHANGE_ID}/specs/[file]
2. Read the merged spec: agentd/specs/[file]
3. Verify ALL changes from delta are present in merged spec
4. Check for hallucinations (content added by Gemini not in delta)
5. Check for omissions (content missing that should be present)

### 2. Format Validation

For each merged spec, verify:
- Proper headings: # Specification:, ## Overview, ## Requirements
- Requirement format: ### R\d+: [title]
- Scenario format: #### Scenario: [name]
- WHEN/THEN clauses present in all scenarios

### 3. CHANGELOG Verification

Check the latest CHANGELOG entry:
- Is it for ${CHANGE_ID}?
- Does it list all affected spec files?
- Is it concise (1-2 sentences)?
- Does it use past tense?
- Is the date correct?

### 4. Cross-File Validation

- All spec files from delta have corresponding merged files
- No duplicate requirement IDs across specs
- All cross-references are valid

### 5. Generate Report

Update the file: agentd/changes/${CHANGE_ID}/ARCHIVE_REVIEW.md

Fill in all sections with your findings:

1. **Status**: Mark ONE checkbox with [x]:
   - [x] APPROVED if all checks pass, no issues found
   - [x] NEEDS_FIX if minor issues (missing WHEN, formatting errors, incomplete CHANGELOG)
   - [x] REJECTED if major issues (missing requirements, corrupted specs, wrong content)

2. **Merged Specs Review**: For each spec file, list checks:
   - [x] Format valid / [ ] Issue: [description]
   - [x] Delta complete / [ ] Issue: [description]
   - [x] No hallucinations / [ ] Issue: [description]

3. **CHANGELOG Review**: List checks for CHANGELOG

4. **Overall Assessment**: Brief summary of findings

5. **Issues Found**: Number each issue with severity (HIGH/MEDIUM/LOW), category, and description:
   1. **HIGH**: specs/file.md - Missing requirement R3 from delta
   2. **MEDIUM**: CHANGELOG doesn't mention session timeout change

6. **Recommendation**: Explain which action to take

**Decision Criteria**:
- **APPROVED**: All checks pass, no issues found
- **NEEDS_FIX**: Minor issues that need fixing before archive
- **REJECTED**: Major issues that require manual intervention

Be strict. Quality is critical. Look for:
- Hallucinations: Content added by Gemini that wasn't in delta
- Omissions: Content from delta missing in merged spec
- Format violations: Incorrect heading levels, malformed requirements
- Logic errors: Contradictions, broken references

Now perform the review and update the ARCHIVE_REVIEW.md file.
EOF
)

cd "$PROJECT_ROOT" && codex exec --full-auto --json "$PROMPT" | while IFS= read -r line; do
  type=$(echo "$line" | jq -r '.type // empty' 2>/dev/null)
  case "$type" in
    item.completed)
      item_type=$(echo "$line" | jq -r '.item.type // empty' 2>/dev/null)
      case "$item_type" in
        reasoning)
          text=$(echo "$line" | jq -r '.item.text // empty' 2>/dev/null)
          [ -n "$text" ] && echo "💭 $text"
          ;;
        command_execution)
          cmd=$(echo "$line" | jq -r '.item.command // empty' 2>/dev/null)
          status=$(echo "$line" | jq -r '.item.status // empty' 2>/dev/null)
          [ -n "$cmd" ] && echo "⚡ $cmd ($status)"
          ;;
        agent_message)
          echo "✅ Archive review complete"
          ;;
      esac
      ;;
    turn.completed)
      tokens=$(echo "$line" | jq -r '.usage.input_tokens // 0' 2>/dev/null)
      echo "📊 Tokens used: $tokens"
      ;;
  esac
done

echo "✅ Review complete: $REVIEW_PATH"
