#!/bin/bash
# Claude resolve script - fixes issues from code review
# Usage: ./claude-resolve.sh <change-id>
#
# Environment variables:
#   AGENTD_MODEL - Model to use: "haiku" (fast), "sonnet" (default), "opus" (deep)
#
set -euo pipefail

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Model selection: default to sonnet
MODEL="${AGENTD_MODEL:-sonnet}"

case "$MODEL" in
    haiku|sonnet|opus) ;;
    *) MODEL="sonnet" ;;
esac

if [ "$MODEL" = "opus" ]; then
    echo "⚠️  Using opus model (high cost)"
fi

echo "🔧 Resolving review issues with Claude ($MODEL): $CHANGE_ID"

cd "$PROJECT_ROOT"

PROMPT=$(cat << PROMPT_END
# Agentd Resolve Reviews Task

Fix issues identified in code review for change: **${CHANGE_ID}**

## Context Files
- Review report: agentd/changes/${CHANGE_ID}/REVIEW.md
- Tasks: agentd/changes/${CHANGE_ID}/tasks.md
- Specs: agentd/changes/${CHANGE_ID}/specs/

## Instructions

1. **Read REVIEW.md** to understand all issues
2. **Fix ALL HIGH severity issues first**:
   - Failing tests
   - Security vulnerabilities
   - Missing features
   - Wrong behavior
3. **Then fix MEDIUM severity issues**:
   - Style inconsistencies
   - Missing tests
   - Architecture concerns
4. **Re-run tests** to verify fixes
5. Update code, tests, and documentation as needed

## Priority Order
1. Security issues (critical)
2. Failing tests (high)
3. Missing features (high)
4. Style/consistency (medium)

**All tests must pass before completion.**
PROMPT_END
)

echo "$PROMPT" | claude --model "$MODEL" --print --output-format text
