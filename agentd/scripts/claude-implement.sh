#!/bin/bash
# Claude implement script - writes code AND tests
# Usage: ./claude-implement.sh <change-id>

CHANGE_ID="$1"
TASKS="${2:-}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🎨 Implementing with Claude: $CHANGE_ID"

PROMPT=$(cat << EOF
# Agentd Implement Task

Implement the proposal for agentd/changes/${CHANGE_ID}/.

## Instructions
1. Read proposal.md, tasks.md, and specs/
2. Implement ALL tasks in tasks.md (or only ${TASKS} if specified)
3. **Write tests for all implemented features** (unit + integration)
   - Test all spec scenarios (WHEN/THEN cases)
   - Include edge cases and error handling
   - Use existing test framework patterns
4. Update IMPLEMENTATION.md with progress notes

## Code Quality
- Follow existing code style and patterns
- Add proper error handling
- Include documentation comments where needed

**IMPORTANT**: Write comprehensive tests. Tests are as important as the code itself.
EOF
)

# This is a placeholder - actual implementation happens via Claude Code Skills
# When called from CLI, Claude IDE will handle the implementation
echo "⚠️  This script is a placeholder - implementation happens via Claude Code Skills"
