# Example Scripts

These are example scripts showing how to integrate AI tools with Specter.

## Setup

1. Copy these scripts to your project:
   ```bash
   cp examples/scripts/* .agentd/scripts/
   ```

2. Make them executable:
   ```bash
   chmod +x .agentd/scripts/*.sh
   ```

3. Customize for your AI tools:
   - Edit each script to use your preferred AI CLI or API
   - Configure API keys in environment variables
   - Adjust prompts and output formats

## Scripts

### gemini-proposal.sh
Generates proposal using Gemini (2M context).

**Options:**
- Use `gemini-cli` if available
- Or call Gemini API directly with curl
- Or integrate with your custom AI tool

### codex-challenge.sh
Analyzes proposal for issues using Codex.

**Options:**
- Use `codex-cli` if available
- Or use Claude Code with custom skill
- Or call OpenAI Codex API

### gemini-reproposal.sh
Refines proposal based on challenge feedback.

Similar to `gemini-proposal.sh` but reads `CHALLENGE.md` as additional context.

### claude-implement.sh
Implements tasks using Claude.

**Options:**
- Use Claude Code CLI
- Or call Claude API via Anthropic SDK
- Or integrate with cursor/cline

### codex-verify.sh
Generates tests and verifies implementation.

**Options:**
- Use `codex-cli` for test generation
- Or use Claude Code for testing
- Or integrate with existing test framework

## Integration Patterns

### 1. CLI Tools (Recommended)
```bash
gemini /openspec:proposal "$CHANGE_ID" "$DESCRIPTION"
codex challenge --proposal "$PROPOSAL_FILE"
claude implement --tasks "$TASKS_FILE"
```

### 2. API Calls
```bash
curl -X POST "https://api.anthropic.com/v1/messages" \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "content-type: application/json" \
  -d '{"model": "claude-3-5-sonnet-20241022", ...}'
```

### 3. IDE Integration
```bash
# Use Cursor API
cursor-api execute-task --file "$TASKS_FILE"

# Use Cline
cline run --config "$CHANGE_DIR/cline.json"
```

## Environment Variables

Create `.env` file in your project:
```bash
# API Keys
ANTHROPIC_API_KEY=sk-ant-...
GEMINI_API_KEY=...
OPENAI_API_KEY=sk-...

# CLI Paths (optional)
GEMINI_CLI=/usr/local/bin/gemini
CODEX_CLI=/usr/local/bin/codex
CLAUDE_CLI=/usr/local/bin/claude
```

Load in scripts:
```bash
source .env
```

## Testing Scripts

Test individual scripts:
```bash
# Test proposal generation
.agentd/scripts/gemini-proposal.sh test-change "Test description"

# Test challenge
.agentd/scripts/codex-challenge.sh test-change

# Check output
ls -la changes/test-change/
```
