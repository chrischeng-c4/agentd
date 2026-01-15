#!/bin/bash
# Gemini CHANGELOG script - generates CHANGELOG entry
# Usage: ./gemini-changelog.sh <change-id>

set -euo pipefail

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "📝 Generating CHANGELOG entry: $CHANGE_ID"

PROMPT=$(cat << EOF
# Agentd Archive: CHANGELOG Generation

Generate a concise CHANGELOG entry for change: ${CHANGE_ID}

## Input Files

Read these files to understand what changed:
- agentd/changes/${CHANGE_ID}/proposal.md
- agentd/changes/${CHANGE_ID}/tasks.md
- agentd/changes/${CHANGE_ID}/specs/ (all spec files)

## Output Format

Use the Keep a Changelog format:

\`\`\`
## $(date +%Y-%m-%d): <Title> (${CHANGE_ID})
<1-2 sentence summary of what changed and why>
- Related specs: spec1.md, spec2.md
\`\`\`

## Requirements

- **1-2 sentences only** - Be concise
- Focus on **what** and **why** (not how)
- Use past tense (e.g., "Added", "Updated", "Fixed")
- List all affected spec files

## Example

\`\`\`
## 2026-01-13: Add OAuth 2.0 Authentication (add-oauth)
Added OAuth 2.0 support with Google and GitHub providers to enable social login. Includes automatic token refresh with 7-day expiry.
- Related specs: specs/auth/oauth.md, specs/auth/session.md
\`\`\`

## Output

Prepend the entry to: agentd/specs/CHANGELOG.md

If CHANGELOG.md doesn't exist, create it with:
\`\`\`
# CHANGELOG

All notable changes to this project's specifications will be documented in this file.

[Your generated entry here]
\`\`\`
EOF
)

echo "$PROMPT" | gemini agentd:changelog -m gemini-3-flash-preview --output-format stream-json
