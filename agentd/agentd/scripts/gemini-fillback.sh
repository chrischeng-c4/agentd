#!/bin/bash
# Gemini fillback (reverse-engineer specs) script
# Usage: ./gemini-fillback.sh <change-id> <json-request>
set -euo pipefail

CHANGE_ID="$1"
JSON_REQUEST="$2"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🤖 Reverse-engineering specs with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by agentd CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/GEMINI.md"

# Parse JSON request
FILE_COUNT=$(echo "$JSON_REQUEST" | jq -r '.files | length')
PROMPT=$(echo "$JSON_REQUEST" | jq -r '.prompt')

echo "📊 Analyzing $FILE_COUNT source files..."

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## Task
${PROMPT}

## Source Files
The following source files have been scanned from the codebase:

EOF
)

# Add file information
for i in $(seq 0 $((FILE_COUNT - 1))); do
    FILE_PATH=$(echo "$JSON_REQUEST" | jq -r ".files[$i].path")
    CONTEXT="${CONTEXT}
- ${FILE_PATH}"
done

CONTEXT="${CONTEXT}

## Source Code
\`\`\`
"

# Add file contents
for i in $(seq 0 $((FILE_COUNT - 1))); do
    FILE_PATH=$(echo "$JSON_REQUEST" | jq -r ".files[$i].path")
    FILE_CONTENT=$(echo "$JSON_REQUEST" | jq -r ".files[$i].content")

    CONTEXT="${CONTEXT}
=== ${FILE_PATH} ===
${FILE_CONTENT}

"
done

CONTEXT="${CONTEXT}\`\`\`

## Instructions
Analyze the provided source code and generate:
1. proposal.md in agentd/changes/${CHANGE_ID}/proposal.md
2. Technical specifications in agentd/changes/${CHANGE_ID}/specs/*.md
3. tasks.md in agentd/changes/${CHANGE_ID}/tasks.md

The specifications should include:
- High-level architecture and design patterns used
- Key requirements and components
- Data models and interfaces
- Acceptance criteria based on code behavior

Focus on creating actionable, well-structured Agentd specifications that capture the technical design.
"

# Call Gemini CLI with pre-defined command
echo "$CONTEXT" | gemini agentd:fillback -m gemini-3-flash-preview --output-format stream-json
