#!/bin/bash
# Codex code review script with test execution and security scanning
# Usage: ./codex-review.sh <change-id> <iteration>
set -euo pipefail

CHANGE_ID="$1"
ITERATION="${2:-0}"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Reviewing code with Codex (Iteration $ITERATION): $CHANGE_ID"

# Step 1: Run tests
echo "🧪 Running tests..."
TEST_OUTPUT=$(cargo test 2>&1 || true)
TEST_STATUS=$?

# Step 2: Run security scans
echo "🔒 Running security scans..."

# Rust: cargo audit (if available)
AUDIT_OUTPUT=""
if command -v cargo-audit &> /dev/null; then
    AUDIT_OUTPUT=$(cargo audit 2>&1 || true)
fi

# Universal: semgrep (if available)
SEMGREP_OUTPUT=""
if command -v semgrep &> /dev/null; then
    SEMGREP_OUTPUT=$(semgrep --config=auto --json 2>&1 || true)
fi

# Clippy with security lints
CLIPPY_OUTPUT=""
CLIPPY_OUTPUT=$(cargo clippy -- -W clippy::all -W clippy::pedantic 2>&1 || true)

# Step 3: Save outputs to temp files for Codex to read
TEMP_DIR=$(mktemp -d)
echo "$TEST_OUTPUT" > "$TEMP_DIR/test_output.txt"
echo "$AUDIT_OUTPUT" > "$TEMP_DIR/audit_output.txt"
echo "$SEMGREP_OUTPUT" > "$TEMP_DIR/semgrep_output.txt"
echo "$CLIPPY_OUTPUT" > "$TEMP_DIR/clippy_output.txt"

# Use change-specific AGENTS.md context
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/agentd/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Agentd Code Review Task (Iteration $ITERATION)

Review the implementation for agentd/changes/${CHANGE_ID}/.

## Available Data
- Test results: $TEMP_DIR/test_output.txt
- Security audit: $TEMP_DIR/audit_output.txt
- Semgrep scan: $TEMP_DIR/semgrep_output.txt
- Clippy output: $TEMP_DIR/clippy_output.txt

## Instructions
1. Read proposal.md, tasks.md, specs/ to understand requirements
2. Read implemented code (search for new/modified files)
3. **Analyze test results** from test_output.txt:
   - Parse test pass/fail status
   - Identify failing tests and reasons
   - Calculate coverage if available
4. **Analyze security scan results**:
   - Parse cargo audit for dependency vulnerabilities
   - Parse semgrep for security patterns
   - Parse clippy for code quality and security warnings
5. Review code quality, best practices, and requirement compliance
6. Fill agentd/changes/${CHANGE_ID}/REVIEW.md with comprehensive findings

## Review Focus
1. **Test Results (HIGH)**: Are all tests passing? Coverage adequate?
2. **Security (HIGH)**: Any vulnerabilities from tools? Security best practices?
3. **Best Practices (HIGH)**: Performance, error handling, style
4. **Requirement Compliance (HIGH)**: Does code match proposal/specs?
5. **Consistency (MEDIUM)**: Does code follow existing patterns?
6. **Test Quality (MEDIUM)**: Are tests comprehensive and well-written?

## Severity Guidelines
- **HIGH**: Failing tests, security vulnerabilities, missing features, wrong behavior
- **MEDIUM**: Style inconsistencies, missing tests, minor improvements
- **LOW**: Suggestions, nice-to-haves

## Verdict Guidelines
- **APPROVED**: All tests pass, no HIGH issues (LOW/MEDIUM issues acceptable)
- **NEEDS_CHANGES**: Some tests fail or HIGH/MEDIUM issues exist (fixable)
- **MAJOR_ISSUES**: Many failing tests or critical security issues

Be thorough but fair. Include iteration number in REVIEW.md.
EOF
)

# Run with JSON streaming
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
          echo "✅ Review analysis complete"
          ;;
      esac
      ;;
    turn.completed)
      tokens=$(echo "$line" | jq -r '.usage.input_tokens // 0' 2>/dev/null)
      echo "📊 Tokens used: $tokens"
      ;;
  esac
done

# Cleanup temp files
rm -rf "$TEMP_DIR"

echo "✅ Review complete (Iteration $ITERATION)"
