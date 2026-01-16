#!/bin/bash
# Claude implement script - writes code AND tests
# Usage: ./claude-implement.sh <change-id> [--tasks "1.1,1.2"]
#
# Environment variables:
#   AGENTD_MODEL - Model to use (e.g., "sonnet", "opus", "haiku")
#
set -euo pipefail

CHANGE_ID="$1"
shift || true
TASKS=""

# Parse optional --tasks argument
while [[ $# -gt 0 ]]; do
    case $1 in
        --tasks)
            TASKS="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Model selection: default to sonnet
MODEL="${AGENTD_MODEL:-sonnet}"

echo "🎨 Implementing with Claude ($MODEL): $CHANGE_ID"

# Build task filter instruction
TASK_FILTER=""
if [ -n "$TASKS" ]; then
    TASK_FILTER="Only implement tasks: ${TASKS}"
fi

PROMPT=$(cat << EOF
# Agentd Implement Task

Implement the proposal for agentd/changes/${CHANGE_ID}/.

## Instructions
1. Read proposal.md, tasks.md, and specs/
2. Implement ALL tasks in tasks.md ${TASK_FILTER}
3. **Write tests for all implemented features** (unit + integration)
   - Test all spec scenarios (WHEN/THEN cases)
   - Include edge cases and error handling
   - Use existing test framework patterns
4. Create/update IMPLEMENTATION.md with progress notes

## Code Quality
- Follow existing code style and patterns
- Add proper error handling
- Include documentation comments where needed

**IMPORTANT**: Write comprehensive tests. Tests are as important as the code itself.
EOF
)

# Run Claude CLI in headless mode
cd "$PROJECT_ROOT"
echo "$PROMPT" | claude -p \
    --model "$MODEL" \
    --allowedTools "Write,Edit,Read,Bash,Glob,Grep" \
    --output-format stream-json \
    --verbose
