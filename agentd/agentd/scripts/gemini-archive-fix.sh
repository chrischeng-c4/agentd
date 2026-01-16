#!/bin/bash
# Gemini archive fix script - fixes issues from archive review
# Usage: ./gemini-archive-fix.sh <change-id>
#
# Environment variables:
#   AGENTD_MODEL - Model to use (e.g., "gemini-2.0-flash")
#
set -euo pipefail

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔧 Fixing archive issues with Gemini: $CHANGE_ID"

cd "$PROJECT_ROOT"

PROMPT=$(cat << PROMPT_END
# Agentd Archive Fix Task

Fix issues identified in archive review for change: **${CHANGE_ID}**

## Context Files
- Archive review report: agentd/changes/${CHANGE_ID}/ARCHIVE_REVIEW.md
- Merged main specs: agentd/specs/
- Change delta specs: agentd/changes/${CHANGE_ID}/specs/

## Instructions

1. **Read ARCHIVE_REVIEW.md** to understand all issues
2. **Fix ALL issues** in the merged specs:
   - Missing content from change specs
   - Duplicate content
   - Format errors
   - Inconsistencies
3. **Verify the merged specs** are complete and correct
4. **Update CHANGELOG** if needed

## Priority
Fix in order: Missing Content -> Format Errors -> Inconsistencies

## Output
Edit the spec files in agentd/specs/ directly to fix the issues.

**Focus on preserving all requirements and scenarios from the change.**
PROMPT_END
)

# Run Gemini CLI
echo "$PROMPT" | gemini agentd:archive-fix -m gemini-3-flash-preview --output-format stream-json
