#!/bin/bash
# Gemini proposal generation script
# Usage: ./gemini-proposal.sh <change-id> <description>
set -euo pipefail

CHANGE_ID="$1"
DESCRIPTION="$2"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🤖 Generating proposal with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by agentd CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## User Request
${DESCRIPTION}

## Instructions
Create proposal files in agentd/changes/${CHANGE_ID}/.
EOF
)

# Call Gemini CLI with pre-defined command
# Use gemini-3-flash-preview model
echo "$CONTEXT" | gemini agentd:proposal -m gemini-3-flash-preview --output-format stream-json
