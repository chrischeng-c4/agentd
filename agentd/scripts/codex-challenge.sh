#!/bin/bash
# Codex challenge script
# Usage: ./codex-challenge.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Analyzing proposal with Codex: $CHANGE_ID"

# Use change-specific AGENTS.md context (generated dynamically by agentd CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Agentd Challenge Task

A skeleton CHALLENGE.md has been created at agentd/changes/${CHANGE_ID}/CHALLENGE.md.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read all proposal files in agentd/changes/${CHANGE_ID}/:
   - proposal.md
   - tasks.md
   - diagrams.md
   - specs/*.md

3. Explore the existing codebase

4. Fill the CHALLENGE.md skeleton with your findings:
   - **Internal Consistency Issues** (HIGH): Check if proposal docs match each other
   - **Code Alignment Issues** (MEDIUM/LOW): Check alignment with existing code
     - If proposal mentions "refactor" or "BREAKING", note deviations as intentional
   - **Quality Suggestions** (LOW): Missing tests, error handling, etc.
   - **Verdict**: APPROVED/NEEDS_REVISION/REJECTED based on HIGH severity count

Be thorough and constructive. Reference specific files and provide actionable recommendations.
EOF
)

# Run non-interactively with full auto mode
codex exec --full-auto "$PROMPT"
