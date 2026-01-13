use crate::{models::SpecterConfig, Result};
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};

// Claude Code Skills
const SKILL_PROPOSAL: &str = include_str!("../../templates/skills/specter-proposal/SKILL.md");
const SKILL_CHALLENGE: &str = include_str!("../../templates/skills/specter-challenge/SKILL.md");
const SKILL_REPROPOSAL: &str = include_str!("../../templates/skills/specter-reproposal/SKILL.md");
const SKILL_IMPLEMENT: &str = include_str!("../../templates/skills/specter-implement/SKILL.md");
const SKILL_VERIFY: &str = include_str!("../../templates/skills/specter-verify/SKILL.md");
const SKILL_FIX: &str = include_str!("../../templates/skills/specter-fix/SKILL.md");
const SKILL_ARCHIVE: &str = include_str!("../../templates/skills/specter-archive/SKILL.md");

// Gemini Commands
const GEMINI_PROPOSAL: &str = include_str!("../../templates/gemini/commands/specter/proposal.toml");
const GEMINI_REPROPOSAL: &str =
    include_str!("../../templates/gemini/commands/specter/reproposal.toml");
const GEMINI_SETTINGS: &str = include_str!("../../templates/gemini/settings.json");

// Codex Prompts
const CODEX_CHALLENGE: &str = include_str!("../../templates/codex/prompts/specter-challenge.md");
const CODEX_VERIFY: &str = include_str!("../../templates/codex/prompts/specter-verify.md");

// AI Context Files (GEMINI.md and AGENTS.md) are now generated dynamically per change
// from templates/GEMINI.md and templates/AGENTS.md

pub async fn run(name: Option<&str>) -> Result<()> {
    let project_root = env::current_dir()?;

    // Check if already initialized
    let specter_dir = project_root.join("specter");
    let claude_dir = project_root.join(".claude");

    if specter_dir.exists() {
        println!("{}", "⚠️  Specter is already initialized".yellow());
        println!("   Run with --force to reinstall");
        return Ok(());
    }

    println!(
        "{}",
        "🎭 Initializing Specter for Claude Code...".cyan().bold()
    );
    println!();

    // Create directory structure
    println!("{}", "📁 Creating directory structure...".cyan());
    std::fs::create_dir_all(&specter_dir)?;
    std::fs::create_dir_all(specter_dir.join("specs"))?;
    std::fs::create_dir_all(specter_dir.join("changes"))?;
    std::fs::create_dir_all(specter_dir.join("archive"))?;
    std::fs::create_dir_all(specter_dir.join("scripts"))?;

    // AI context files (GEMINI.md, AGENTS.md) are now generated dynamically
    // per change in specter/changes/<change-id>/ by the CLI commands

    // Create Claude Code skills directory
    let skills_dir = claude_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    // Create config
    let mut config = SpecterConfig::default();
    if let Some(n) = name {
        config.project_name = n.to_string();
    } else if let Some(dir_name) = project_root.file_name() {
        config.project_name = dir_name.to_string_lossy().to_string();
    }
    config.scripts_dir = specter_dir.join("scripts");
    config.save(&project_root)?;

    // Install Claude Code Skills
    println!("{}", "🤖 Installing Claude Code Skills...".cyan());
    install_claude_skills(&skills_dir)?;

    // Install Gemini Commands (project-specific)
    println!();
    println!("{}", "🤖 Installing Gemini Commands...".cyan());
    let gemini_dir = claude_dir.parent().unwrap().join(".gemini");
    std::fs::create_dir_all(gemini_dir.join("commands/specter"))?;
    install_gemini_commands(&gemini_dir)?;

    // Install Codex Prompts (user-space)
    println!();
    println!("{}", "🤖 Installing Codex Prompts...".cyan());
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let codex_prompts_dir = PathBuf::from(home_dir).join(".codex/prompts");
    install_codex_prompts(&codex_prompts_dir)?;

    // Create helper scripts
    create_helper_scripts(&specter_dir.join("scripts"))?;

    println!();
    println!("{}", "✅ Specter initialized successfully!".green().bold());
    println!();
    println!("{}", "📁 Structure:".cyan());
    println!("   specter/                   - Main Specter directory");
    println!("   specter/specs/             - Main specifications");
    println!("   specter/changes/           - Active changes");
    println!("   specter/archive/           - Completed changes");
    println!("   .claude/skills/            - 7 Skills installed");
    println!("   .gemini/commands/specter/  - 2 Gemini commands");
    println!("   ~/.codex/prompts/          - 2 Codex prompts");
    println!();

    println!("{}", "🤖 AI Commands Installed:".cyan().bold());
    println!(
        "   {} - Proposal generation",
        "gemini specter:proposal".green()
    );
    println!(
        "   {} - Proposal refinement",
        "gemini specter:reproposal".green()
    );
    println!("   {} - Code review", "codex specter-challenge".green());
    println!("   {} - Test generation", "codex specter-verify".green());
    println!();

    println!(
        "{}",
        "🎯 Available Skills (use in Claude Code):".cyan().bold()
    );
    println!(
        "   {} - Generate proposal with Gemini",
        "/specter:proposal".green()
    );
    println!(
        "   {} - Challenge proposal with Codex",
        "/specter:challenge".green()
    );
    println!(
        "   {} - Refine based on feedback",
        "/specter:reproposal".green()
    );
    println!(
        "   {} - Implement with Claude",
        "/specter:implement".green()
    );
    println!("   {} - Verify with tests", "/specter:verify".green());
    println!("   {} - Fix verification failures", "/specter:fix".green());
    println!(
        "   {} - Archive completed change",
        "/specter:archive".green()
    );
    println!();

    println!("{}", "⏭️  Next Steps:".yellow().bold());
    println!("   1. In Claude Code, run:");
    println!(
        "      {}",
        "/specter:proposal my-feature \"Add awesome feature\"".cyan()
    );
    println!();
    println!("   2. Configure API keys (optional):");
    println!("      Edit specter/scripts/config.sh");
    println!();
    println!("   3. Read the guide:");
    println!("      cat specter/README.md");

    Ok(())
}

fn install_claude_skills(skills_dir: &Path) -> Result<()> {
    let skills = vec![
        ("proposal", SKILL_PROPOSAL),
        ("challenge", SKILL_CHALLENGE),
        ("reproposal", SKILL_REPROPOSAL),
        ("implement", SKILL_IMPLEMENT),
        ("verify", SKILL_VERIFY),
        ("fix", SKILL_FIX),
        ("archive", SKILL_ARCHIVE),
    ];

    for (name, content) in skills {
        let skill_dir = skills_dir.join(format!("specter-{}", name));
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(skill_dir.join("SKILL.md"), content)?;
        println!("   ✓ specter-{}", name);
    }

    Ok(())
}

fn create_helper_scripts(scripts_dir: &Path) -> Result<()> {
    // Create updated script templates that use AI CLI commands
    let gemini_proposal = r#"#!/bin/bash
# Gemini proposal generation script
# Usage: ./gemini-proposal.sh <change-id> <description>
set -euo pipefail

CHANGE_ID="$1"
DESCRIPTION="$2"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🤖 Generating proposal with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by specter CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/specter/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## User Request
${DESCRIPTION}

## Instructions
Create proposal files in specter/changes/${CHANGE_ID}/.
EOF
)

# Call Gemini CLI with pre-defined command
echo "$CONTEXT" | gemini specter:proposal --output-format stream-json
"#;

    let gemini_reproposal = r#"#!/bin/bash
# Gemini reproposal script
# Usage: ./gemini-reproposal.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root (parent of scripts dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔄 Refining proposal with Gemini: $CHANGE_ID"

# Use change-specific GEMINI.md context (generated dynamically by specter CLI)
export GEMINI_SYSTEM_MD="$PROJECT_ROOT/specter/changes/$CHANGE_ID/GEMINI.md"

# Build context for Gemini
CONTEXT=$(cat << EOF
## Change ID
${CHANGE_ID}

## Instructions
Read specter/changes/${CHANGE_ID}/CHALLENGE.md and fix all HIGH and MEDIUM severity issues.
EOF
)

# Call Gemini CLI with pre-defined command
# Use --resume latest to reuse the proposal session (cached codebase context)
echo "$CONTEXT" | gemini specter:reproposal --resume latest --output-format stream-json
"#;

    let codex_challenge = r#"#!/bin/bash
# Codex challenge script
# Usage: ./codex-challenge.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Analyzing proposal with Codex: $CHANGE_ID"

# Use change-specific AGENTS.md context (generated dynamically by specter CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/specter/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Specter Challenge Task

A skeleton CHALLENGE.md has been created at specter/changes/${CHANGE_ID}/CHALLENGE.md.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read all proposal files in specter/changes/${CHANGE_ID}/:
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
"#;

    let codex_review = r#"#!/bin/bash
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
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/specter/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Specter Code Review Task (Iteration $ITERATION)

Review the implementation for specter/changes/${CHANGE_ID}/.

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
6. Fill specter/changes/${CHANGE_ID}/REVIEW.md with comprehensive findings

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

# Run non-interactively with full auto mode
codex exec --full-auto "$PROMPT"

# Cleanup temp files
rm -rf "$TEMP_DIR"

echo "✅ Review complete (Iteration $ITERATION)"
"#;

    // Create codex-rechallenge.sh for session resumption
    let codex_rechallenge = r#"#!/bin/bash
# Codex re-challenge script (resumes previous session for cached context)
# Usage: ./codex-rechallenge.sh <change-id>
set -euo pipefail

CHANGE_ID="$1"

# Get the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔍 Re-analyzing proposal with Codex (resuming session): $CHANGE_ID"

# Use change-specific AGENTS.md context (generated dynamically by specter CLI)
# Note: Set CODEX_INSTRUCTIONS_FILE if your Codex CLI supports it
export CODEX_INSTRUCTIONS_FILE="$PROJECT_ROOT/specter/changes/$CHANGE_ID/AGENTS.md"

# Build prompt with context
PROMPT=$(cat << EOF
# Specter Re-Challenge Task

A skeleton CHALLENGE.md has been updated at specter/changes/${CHANGE_ID}/CHALLENGE.md.
The proposal has been revised based on previous feedback.

## Instructions
1. Read the skeleton CHALLENGE.md to understand the structure

2. Read the UPDATED proposal files in specter/changes/${CHANGE_ID}/:
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
"#;

    std::fs::write(scripts_dir.join("gemini-proposal.sh"), gemini_proposal)?;
    std::fs::write(scripts_dir.join("gemini-reproposal.sh"), gemini_reproposal)?;
    std::fs::write(scripts_dir.join("codex-challenge.sh"), codex_challenge)?;
    std::fs::write(scripts_dir.join("codex-rechallenge.sh"), codex_rechallenge)?;
    std::fs::write(scripts_dir.join("codex-review.sh"), codex_review)?;

    // Updated claude-implement.sh with emphasis on test writing
    let claude_implement = r#"#!/bin/bash
# Claude implement script - writes code AND tests
# Usage: ./claude-implement.sh <change-id>

CHANGE_ID="$1"
TASKS="${2:-}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🎨 Implementing with Claude: $CHANGE_ID"

PROMPT=$(cat << EOF
# Specter Implement Task

Implement the proposal for specter/changes/${CHANGE_ID}/.

## Instructions
1. Read proposal.md, tasks.md, diagrams.md, and specs/
2. Implement ALL tasks in tasks.md (or only ${TASKS} if specified)
3. **Write tests for all implemented features** (unit + integration)
   - Test all spec scenarios (WHEN/THEN cases)
   - Include edge cases and error handling
   - Use existing test framework patterns
4. Update IMPLEMENTATION.md with progress notes

## Code Quality
- Follow existing code style and patterns
- Add proper error handling
- Include documentation comments where needed

**IMPORTANT**: Write comprehensive tests. Tests are as important as the code itself.
EOF
)

# This is a placeholder - actual implementation happens via Claude Code Skills
# When called from CLI, Claude IDE will handle the implementation
echo "⚠️  This script is a placeholder - implementation happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-implement.sh"), claude_implement)?;

    // claude-resolve.sh for fixing review issues
    let claude_resolve = r#"#!/bin/bash
# Claude resolve script - fixes issues from code review
# Usage: ./claude-resolve.sh <change-id>

CHANGE_ID="$1"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔧 Resolving review issues with Claude: $CHANGE_ID"

PROMPT=$(cat << EOF
# Specter Resolve Reviews Task

Fix issues identified in code review for specter/changes/${CHANGE_ID}/.

## Instructions
1. Read REVIEW.md to understand all issues
2. Fix ALL HIGH and MEDIUM severity issues:
   - Failing tests
   - Security vulnerabilities
   - Missing features
   - Wrong behavior
   - Style inconsistencies
   - Missing tests
3. Update code, tests, and documentation as needed
4. Update IMPLEMENTATION.md with resolution notes

Focus on HIGH severity issues first, then MEDIUM.
EOF
)

# This is a placeholder - actual resolution happens via Claude Code Skills
echo "⚠️  This script is a placeholder - resolution happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-resolve.sh"), claude_resolve)?;

    // Placeholder for claude-fix.sh (kept for backward compatibility)
    let claude_fix = r#"#!/bin/bash
# Claude fix script
# Usage: ./claude-fix.sh <change-id>

CHANGE_ID="$1"

echo "🔧 Fixing issues: $CHANGE_ID"
echo "⚠️  This script is a placeholder - fixing happens via Claude Code Skills"
"#;
    std::fs::write(scripts_dir.join("claude-fix.sh"), claude_fix)?;

    // Make scripts executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in &[
            "gemini-proposal.sh",
            "gemini-reproposal.sh",
            "codex-challenge.sh",
            "codex-rechallenge.sh",
            "codex-review.sh",
            "claude-implement.sh",
            "claude-resolve.sh",
            "claude-fix.sh",
        ] {
            let path = scripts_dir.join(script);
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }
    }

    Ok(())
}

fn install_gemini_commands(gemini_dir: &Path) -> Result<()> {
    let commands_dir = gemini_dir.join("commands/specter");

    // Install command definitions
    std::fs::write(commands_dir.join("proposal.toml"), GEMINI_PROPOSAL)?;
    std::fs::write(commands_dir.join("reproposal.toml"), GEMINI_REPROPOSAL)?;

    println!("   ✓ gemini specter:proposal");
    println!("   ✓ gemini specter:reproposal");

    // Install settings
    std::fs::write(gemini_dir.join("settings.json"), GEMINI_SETTINGS)?;
    println!("   ✓ settings.json");

    Ok(())
}

fn install_codex_prompts(prompts_dir: &Path) -> Result<()> {
    // Create directory if it doesn't exist
    std::fs::create_dir_all(prompts_dir)?;

    let challenge_path = prompts_dir.join("specter-challenge.md");
    let verify_path = prompts_dir.join("specter-verify.md");

    // Check if prompts already exist
    let challenge_exists = challenge_path.exists();
    let verify_exists = verify_path.exists();

    if challenge_exists || verify_exists {
        println!();
        println!(
            "   {} Codex prompts already exist in ~/.codex/prompts/",
            "⚠️".yellow()
        );

        // Try to use interactive prompt, fall back to default if not available
        use dialoguer::Confirm;
        let overwrite = match Confirm::new()
            .with_prompt("Overwrite existing prompts?")
            .default(false)
            .interact()
        {
            Ok(response) => response,
            Err(_) => {
                // Not a terminal or dialoguer failed - use default (don't overwrite)
                println!("   (non-interactive mode: keeping existing prompts)");
                false
            }
        };

        if !overwrite {
            println!("   Skipping Codex prompts installation");
            return Ok(());
        }
    }

    // Install prompt files
    std::fs::write(&challenge_path, CODEX_CHALLENGE)?;
    std::fs::write(&verify_path, CODEX_VERIFY)?;

    println!("   ✓ codex specter-challenge (installed to ~/.codex/prompts/)");
    println!("   ✓ codex specter-verify (installed to ~/.codex/prompts/)");

    Ok(())
}
