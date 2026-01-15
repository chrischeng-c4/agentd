#!/bin/bash
# Claude fix script - fixes issues from verification
# Usage: ./claude-fix.sh <change-id>
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

echo "🔧 Fixing issues with Claude ($MODEL): $CHANGE_ID"

cd "$PROJECT_ROOT"

PROMPT=$(cat << PROMPT_END
# Agentd Fix Task

Fix issues identified in verification for change: **${CHANGE_ID}**

## Context Files
- Verification report: agentd/changes/${CHANGE_ID}/VERIFICATION.md
- Implementation: agentd/changes/${CHANGE_ID}/

## Instructions

1. **Read VERIFICATION.md** to understand all issues
2. **Fix ALL issues** identified:
   - Failing tests
   - Security vulnerabilities
   - Missing edge cases
   - Incorrect behavior
3. **Re-run affected tests** to verify fixes
4. Update code and tests as needed

## Priority
Fix in order: Security issues → Failing tests → Other issues

**Focus on making all tests pass.**
PROMPT_END
)

echo "$PROMPT" | claude --model "$MODEL" --print --output-format text
