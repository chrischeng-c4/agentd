#!/bin/bash
# Claude implement script - writes code AND tests
# Usage: ./claude-implement.sh <change-id> [--tasks=1.1,2.1]
#
# Environment variables:
#   AGENTD_MODEL - Model to use: "haiku" (fast), "sonnet" (default), "opus" (deep)
#
set -euo pipefail

CHANGE_ID="$1"
TASKS="${2:-}"

# Get the project root (parent of agentd dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Model selection: default to sonnet, can override via AGENTD_MODEL
MODEL="${AGENTD_MODEL:-sonnet}"

# Validate model choice
case "$MODEL" in
    haiku|sonnet|opus)
        ;;
    *)
        echo "⚠️  Unknown model '$MODEL', defaulting to sonnet"
        MODEL="sonnet"
        ;;
esac

# Warn if using opus (expensive)
if [ "$MODEL" = "opus" ]; then
    echo "⚠️  Using opus model (high cost) - use only for complex implementations"
fi

echo "🎨 Implementing with Claude ($MODEL): $CHANGE_ID"

# Change to project root so Claude has correct context
cd "$PROJECT_ROOT"

# Build the task filter instruction
TASK_FILTER=""
if [ -n "$TASKS" ]; then
    TASK_FILTER="Only implement the following tasks: ${TASKS}"
else
    TASK_FILTER="Implement ALL tasks in tasks.md"
fi

# Build the prompt
PROMPT=$(cat << 'PROMPT_END'
# Agentd Implementation Task

You are implementing a proposal for change: **CHANGE_ID_PLACEHOLDER**

## Context Files
- Proposal: agentd/changes/CHANGE_ID_PLACEHOLDER/proposal.md
- Tasks: agentd/changes/CHANGE_ID_PLACEHOLDER/tasks.md
- Specs: agentd/changes/CHANGE_ID_PLACEHOLDER/specs/

## Instructions

1. **Read the proposal files first** - understand what needs to be done
2. **TASK_FILTER_PLACEHOLDER**
3. **Write tests** for all implemented features
   - Cover all WHEN/THEN scenarios from specs
   - Include edge cases and error handling
   - Follow existing test patterns in the codebase
4. **Mark completed tasks** in tasks.md: `[ ]` → `[x]`

## Code Quality Requirements
- Follow existing code style and patterns in this codebase
- Add proper error handling (use anyhow for Rust, etc.)
- Include documentation comments for public APIs
- Keep changes minimal and focused on the task

## Output
When done, provide a brief summary of what was implemented.

**IMPORTANT**: Tests are as important as the code itself. Every feature needs tests.
PROMPT_END
)

# Replace placeholders
PROMPT="${PROMPT//CHANGE_ID_PLACEHOLDER/$CHANGE_ID}"
PROMPT="${PROMPT//TASK_FILTER_PLACEHOLDER/$TASK_FILTER}"

# Call Claude CLI in headless mode
echo "$PROMPT" | claude --model "$MODEL" --print --output-format text
