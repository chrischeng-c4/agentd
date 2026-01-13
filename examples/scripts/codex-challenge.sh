#!/bin/bash
# Example Codex challenge script
# This is a template - customize for your needs

set -e

CHANGE_ID="$1"

if [ -z "$CHANGE_ID" ]; then
    echo "Usage: $0 <change-id>"
    exit 1
fi

CHANGE_DIR="changes/$CHANGE_ID"

if [ ! -d "$CHANGE_DIR" ]; then
    echo "❌ Error: Change '$CHANGE_ID' not found"
    exit 1
fi

echo "Challenging proposal: $CHANGE_ID"

# Example: Using codex-cli (if available)
if command -v codex &> /dev/null; then
    codex challenge \
        --proposal "$CHANGE_DIR/proposal.md" \
        --tasks "$CHANGE_DIR/tasks.md" \
        --specs "$CHANGE_DIR/specs" \
        --output "$CHANGE_DIR/CHALLENGE.md"
else
    echo "⚠️  codex-cli not found"
    echo ""
    echo "Alternative: Use Claude Code with custom skill"
    echo ""

    # Example: Call Claude Code via CLI
    # claude code analyze-proposal "$CHANGE_DIR" > "$CHANGE_DIR/CHALLENGE.md"

    echo "❌ Script not fully configured. Please edit:"
    echo "   .agentd/scripts/codex-challenge.sh"
    exit 1
fi

# Verify CHALLENGE.md was created
if [ ! -f "$CHANGE_DIR/CHALLENGE.md" ]; then
    echo "❌ Error: CHALLENGE.md not created"
    exit 1
fi

echo "✅ Challenge report generated"
