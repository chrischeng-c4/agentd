#!/bin/bash
# Claude implement script - writes code AND tests
# Usage: ./claude-implement.sh <change-id> [tasks]
#
# Environment variables:
#   AGENTD_MODEL - Model to use: "haiku" (fast), "sonnet" (default), "opus" (deep)
#
set -euo pipefail

CHANGE_ID="$1"
TASKS="${2:-}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Model selection: default to sonnet
MODEL="${AGENTD_MODEL:-sonnet}"

case "$MODEL" in
    haiku|sonnet|opus) ;;
    *) MODEL="sonnet" ;;
esac

if [ "$MODEL" = "opus" ]; then
    echo "Warning: Using opus model (high cost)"
fi

echo "Implementing with Claude ($MODEL): $CHANGE_ID"

cd "$PROJECT_ROOT"

TASK_FILTER=""
if [ -n "$TASKS" ]; then
    TASK_FILTER="Only implement tasks: $TASKS"
fi

PROMPT=$(cat << PROMPT_END
# Agentd Implementation Task

Implement the proposal for change: **${CHANGE_ID}**

## Context Files
- Proposal: agentd/changes/${CHANGE_ID}/proposal.md
- Tasks: agentd/changes/${CHANGE_ID}/tasks.md
- Specs: agentd/changes/${CHANGE_ID}/specs/

## Instructions

1. **Read all context files** to understand the requirements
2. **Implement ALL tasks** in tasks.md ${TASK_FILTER}
3. **Write tests** for all implemented features:
   - Test all spec scenarios (WHEN/THEN cases)
   - Include edge cases and error handling
   - Use existing test framework patterns
4. **Follow existing code patterns** in the codebase
5. Mark completed tasks in tasks.md: [ ] → [x]

## Code Quality
- Follow existing code style and patterns
- Add proper error handling with anyhow
- Include documentation comments where needed

**IMPORTANT**: Write comprehensive tests. Tests are as important as the code itself.
PROMPT_END
)

# Run Claude in headless mode with specific tool permissions
echo "$PROMPT" | claude --model "$MODEL" \
    --allowedTools "Write,Edit,Read,Bash,Glob,Grep" \
    --print \
    --output-format text
