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
   - specs/*.md (revised - with Mermaid diagrams, JSON Schema, interfaces, acceptance criteria)

3. Re-fill the CHALLENGE.md with your findings:
   - **Internal Consistency Issues** (HIGH): Check if revised proposal docs now match each other
   - **Code Alignment Issues** (MEDIUM/LOW): Check alignment with existing code
     - If proposal mentions "refactor" or "BREAKING", note deviations as intentional
   - **Quality Suggestions** (LOW): Missing tests, error handling, etc.
   - **Verdict**: APPROVED/NEEDS_REVISION/REJECTED based on HIGH severity count

Focus on whether the previous issues have been addressed.
EOF
)

# Run with JSON streaming (codex resume doesn't support --json, use exec instead)
cd "$PROJECT_ROOT" && codex exec --full-auto --json "$PROMPT" | while IFS= read -r line; do
  type=$(echo "$line" | jq -r '.type // empty' 2>/dev/null)
  case "$type" in
    item.completed)
      item_type=$(echo "$line" | jq -r '.item.type // empty' 2>/dev/null)
      case "$item_type" in
        reasoning)
          text=$(echo "$line" | jq -r '.item.text // empty' 2>/dev/null)
          [ -n "$text" ] && echo "💭 $text"
          ;;
        command_execution)
          cmd=$(echo "$line" | jq -r '.item.command // empty' 2>/dev/null)
          status=$(echo "$line" | jq -r '.item.status // empty' 2>/dev/null)
          [ -n "$cmd" ] && echo "⚡ $cmd ($status)"
          ;;
        agent_message)
          echo "✅ Re-challenge analysis complete"
          ;;
      esac
      ;;
    turn.completed)
      tokens=$(echo "$line" | jq -r '.usage.input_tokens // 0' 2>/dev/null)
      echo "📊 Tokens used: $tokens"
      ;;
  esac
done
