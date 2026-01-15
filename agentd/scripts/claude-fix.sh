#!/bin/bash
# Claude fix script - fixes issues from verification
# Usage: ./claude-fix.sh <change-id>
#
# Environment variables:
#   AGENTD_MODEL - Model to use (e.g., "sonnet", "opus", "haiku")
#
set -euo pipefail

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Model selection: default to sonnet
MODEL="${AGENTD_MODEL:-sonnet}"

echo "🔧 Fixing verification issues with Claude ($MODEL): $CHANGE_ID"

PROMPT=$(cat << EOF
# Agentd Fix Task

Fix issues identified during verification for agentd/changes/${CHANGE_ID}/.

## Instructions
1. Read REVIEW.md and STATE.yaml to understand verification issues
2. Fix ALL failing tests and verification errors:
   - Build errors
   - Test failures
   - Type errors
   - Lint warnings
3. Run tests to verify fixes work
4. Update IMPLEMENTATION.md with fix notes

## Code Quality
- Don't break existing functionality
- Ensure all tests pass after fixes
- Follow existing code style
EOF
)

# Run Claude CLI in headless mode
cd "$PROJECT_ROOT"
echo "$PROMPT" | claude -p \
    --model "$MODEL" \
    --allowedTools "Write,Edit,Read,Bash,Glob,Grep" \
    --output-format stream-json
