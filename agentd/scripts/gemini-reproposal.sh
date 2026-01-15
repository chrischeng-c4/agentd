#!/bin/bash
# Gemini reproposal script
# Usage: ./gemini-reproposal.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔄 Refining proposal with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by agentd CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## Instructions
Read agentd/changes/${CHANGE_ID}/CHALLENGE.md and fix all HIGH and MEDIUM severity issues.
EOF
)

# Call Gemini CLI with pre-defined command
# Use --resume latest to reuse the proposal session (cached codebase context)
# Use gemini-3-flash-preview model (has more quota)
echo "$CONTEXT" | gemini agentd:reproposal --resume latest -m gemini-3-flash-preview --output-format stream-json
