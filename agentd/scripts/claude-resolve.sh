#!/bin/bash
# Claude resolve script - fixes issues from code review
# Usage: ./claude-resolve.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔧 Resolving review issues with Claude: $CHANGE_ID"

PROMPT=$(cat << EOF
# Agentd Resolve Reviews Task

Fix issues identified in code review for agentd/changes/${CHANGE_ID}/.

## Instructions
1. Read REVIEW.md to understand all issues
2. Fix ALL HIGH and MEDIUM severity issues:
   - Failing tests
   - Security vulnerabilities
   - Missing features
   - Wrong behavior
   - Style inconsistencies
   - Missing tests
3. Update code, tests, and documentation as needed
4. Update IMPLEMENTATION.md with resolution notes

Focus on HIGH severity issues first, then MEDIUM.
EOF
)

# Run Claude CLI in headless mode
cd "$PROJECT_ROOT"
echo "$PROMPT" | claude -p \
    --allowedTools "Write,Edit,Read,Bash,Glob,Grep" \
    --output-format stream-json
