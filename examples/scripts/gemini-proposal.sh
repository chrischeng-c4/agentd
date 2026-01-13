#!/bin/bash
# Example Gemini proposal generation script
# This is a template - customize for your needs

set -e

CHANGE_ID="$1"
DESCRIPTION="$2"

if [ -z "$CHANGE_ID" ] || [ -z "$DESCRIPTION" ]; then
    echo "Usage: $0 <change-id> <description>"
    exit 1
fi

# Create change directory
CHANGE_DIR="changes/$CHANGE_ID"
mkdir -p "$CHANGE_DIR/specs"

echo "Generating proposal for: $CHANGE_ID"
echo "Description: $DESCRIPTION"

# Example 1: Using gemini-cli (if available)
if command -v gemini &> /dev/null; then
    gemini /openspec:proposal "$CHANGE_ID" "$DESCRIPTION" \
        --context "$CHANGE_DIR" \
        --output-format stream-json
else
    echo "⚠️  gemini-cli not found. Install from: https://geminicli.com"
    echo ""
    echo "Alternative: Call Gemini API directly"
    echo ""

    # Example 2: Using Gemini API directly (placeholder)
    # Uncomment and configure:
    #
    # curl -X POST "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$GEMINI_API_KEY" \
    #   -H 'Content-Type: application/json' \
    #   -d "{
    #     \"contents\": [{
    #       \"parts\": [{
    #         \"text\": \"Generate OpenSpec proposal for: $DESCRIPTION\\n\\nChange ID: $CHANGE_ID\\n\\n[Add your prompt here]\"
    #       }]
    #     }]
    #   }" | jq -r '.candidates[0].content.parts[0].text' > "$CHANGE_DIR/proposal.md"

    echo "❌ Script not fully configured. Please edit:"
    echo "   .agentd/scripts/gemini-proposal.sh"
    exit 1
fi

# Verify files were created
if [ ! -f "$CHANGE_DIR/proposal.md" ]; then
    echo "❌ Error: proposal.md not created"
    exit 1
fi

echo "✅ Proposal generated successfully"
