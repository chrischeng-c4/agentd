#!/bin/bash
# Codex re-challenge script (resumes previous session for cached context)
# Usage: ./codex-rechallenge.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Re-analyzing proposal with Codex (resuming session): $CHANGE_ID"

# Use change-specific AGENTS.md context (generated dynamically by agentd CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Agentd Re-Challenge Task

A skeleton CHALLENGE.md has been updated at agentd/changes/${CHANGE_ID}/CHALLENGE.md.
The proposal has been revised based on previous feedback.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read the UPDATED proposal files in agentd/changes/${CHANGE_ID}/:
   - proposal.md (revised)
   - tasks.md (revised)
   - diagrams.md (revised)
   - specs/*.md (revised)

3. Re-fill the CHALLENGE.md with your findings:
   - **Internal Consistency Issues** (HIGH): Check if revised proposal docs now match each other
   - **Code Alignment Issues** (MEDIUM/LOW): Check alignment with existing code
     - If proposal mentions "refactor" or "BREAKING", note deviations as intentional
   - **Quality Suggestions** (LOW): Missing tests, error handling, etc.
   - **Verdict**: APPROVED/NEEDS_REVISION/REJECTED based on HIGH severity count

Focus on whether the previous issues have been addressed.
EOF
)

# Resume the previous session to reuse cached codebase context
codex resume --last "$PROMPT"
