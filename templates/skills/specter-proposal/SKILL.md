---
name: specter-proposal
description: Generate spec-driven proposal using Gemini (2M context)
user-invocable: true
---

# /specter:proposal - Generate Proposal

Generate a comprehensive spec-driven proposal using Gemini's 2M context window for deep codebase exploration.

## Usage

```
/specter:proposal <change-id> "<description>"
```

**Example**:
```
/specter:proposal add-oauth "Add OAuth authentication with Google and GitHub"
```

## What This Skill Does

1. **Explore Codebase** - Uses Gemini (2M context) to analyze existing code
2. **Generate Proposal** - Creates detailed proposal with:
   - `proposal.md` - Why, what, impact analysis
   - `tasks.md` - Step-by-step implementation checklist
   - `diagrams.md` - 4 architecture diagrams (Mermaid)
   - `specs/<capability>/spec.md` - Requirements with WHEN/THEN scenarios
3. **Validate Structure** - Ensures all required files are created

## Steps

### 1. Validate Input

Check that change-id and description are provided:
```bash
if [ -z "$CHANGE_ID" ]; then
    echo "❌ Error: change-id is required"
    echo "Usage: /specter:proposal <change-id> \"<description>\""
    exit 1
fi
```

### 2. Check if Change Already Exists

```bash
CHANGE_DIR="changes/$CHANGE_ID"
if [ -d "$CHANGE_DIR" ]; then
    echo "⚠️  Change '$CHANGE_ID' already exists"
    echo "Use /specter:refine to update it"
    exit 1
fi
```

### 3. Create Directory Structure

```bash
mkdir -p "$CHANGE_DIR/specs"
```

### 4. Call Gemini for Proposal Generation

**Option A: Using gemini-cli** (recommended if installed):
```bash
if command -v gemini &> /dev/null; then
    gemini /openspec:proposal "$CHANGE_ID" "$DESCRIPTION" \
        --context "$CHANGE_DIR" \
        --output-format stream-json
else
    echo "⚠️  gemini-cli not found"
    # Fall back to Option B
fi
```

**Option B: Direct Gemini API call**:
```bash
# Read project context
PROJECT_CONTEXT=$(cat CLAUDE.md AGENTS.md 2>/dev/null || echo "")

# Call Gemini API
curl -X POST \
  "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=$GEMINI_API_KEY" \
  -H 'Content-Type: application/json' \
  -d "{
    \"contents\": [{
      \"parts\": [{
        \"text\": \"$PROPOSAL_PROMPT\"
      }]
    }],
    \"generationConfig\": {
      \"temperature\": 0.7,
      \"maxOutputTokens\": 8192
    }
  }" | jq -r '.candidates[0].content.parts[0].text' > "$CHANGE_DIR/proposal.md"
```

**Option C: Use Claude (you!) as fallback**:

If Gemini is not available, you should:
1. Read existing specs in `specs/`
2. Explore relevant code files
3. Generate the proposal files yourself
4. Follow the OpenSpec format for requirements and scenarios

### 5. Verify Files Were Created

```bash
REQUIRED_FILES=("proposal.md" "tasks.md" "diagrams.md")
for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$CHANGE_DIR/$file" ]; then
        echo "❌ Error: $file was not created"
        exit 1
    fi
done

# Check for at least one spec file
if ! ls "$CHANGE_DIR/specs"/**/spec.md >/dev/null 2>&1; then
    echo "⚠️  Warning: No spec files found in $CHANGE_DIR/specs/"
fi
```

### 6. Display Success Message

```
✅ Proposal generated successfully!

📄 Files created:
   • changes/{change-id}/proposal.md
   • changes/{change-id}/tasks.md
   • changes/{change-id}/diagrams.md
   • changes/{change-id}/specs/<capability>/spec.md

⏭️  Next steps:
   1. Review the proposal
   2. Run: /specter:challenge {change-id}
```

## Environment Variables Required

```bash
# Option 1: Gemini CLI (recommended)
# Install: npm install -g gemini-cli

# Option 2: Direct API
GEMINI_API_KEY=your-api-key-here

# Optional: Project context
# Create CLAUDE.md and AGENTS.md in project root
```

## Proposal Format Example

**proposal.md**:
```markdown
# Proposal: {Change ID}

## Why
[Why this change is needed]

## What
[What will be implemented]

## Impact
- Files modified: 5
- New files: 3
- Risk level: Medium
```

**tasks.md**:
```markdown
# Implementation Tasks

## Backend
- [ ] 1.1 Create OAuth provider interface
- [ ] 1.2 Implement Google OAuth
- [ ] 1.3 Implement GitHub OAuth

## Frontend
- [ ] 2.1 Add OAuth login buttons
- [ ] 2.2 Handle OAuth callback
```

**specs/auth/spec.md**:
```markdown
## ADDED Requirements

### Requirement: OAuth Authentication
The system SHALL support OAuth authentication with multiple providers.

#### Scenario: User logs in with Google
- **WHEN** user clicks "Login with Google" button
- **THEN** system redirects to Google OAuth page
- **AND** stores OAuth token after successful authentication
```

## Troubleshooting

### Issue: Gemini not found
**Solution**: Install gemini-cli or set GEMINI_API_KEY

### Issue: Empty proposal generated
**Solution**: Check API key and network connection

### Issue: Invalid spec format
**Solution**: Review generated files and manually fix format issues

## Cost Estimate

- **Gemini (2M context)**: ~$0.50 per proposal
- **Claude fallback**: ~$2.00 per proposal

Using Gemini saves ~75% compared to Claude-only approach.
