# Specter - Spec-driven Development Orchestrator 🎭

**Specter** = **Spec** + Orches**ter** (Orchestrator)

A Rust-powered spec-driven development tool with **iterative proposal refinement** through AI orchestration.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🎯 Core Concept

Specter orchestrates **three AI tools** to enable cost-effective, high-quality spec-driven development:

- 🤖 **Gemini CLI** (2M context, low cost) - Code exploration & proposal generation
- 🔍 **Codex CLI** (code specialist) - Challenge proposals & generate tests
- 🎨 **Claude Code** (high quality) - Precise implementation

## ✨ Key Innovations

1. **Challenge Phase** - AI automatically reviews proposals (Codex analyzes against existing code)
2. **Iterative Refinement** - proposal → challenge → reproposal loop until satisfied
3. **Automated Verification** - Codex generates tests from specs and verifies implementation

## 🚀 Quick Start

### Installation

**Option 1: Install from source (Recommended)**

```bash
# Clone and install
git clone https://github.com/your-repo/specter
cd specter
cargo install --path .

# Verify installation
specter --version
```

**Option 2: One-line install script**

```bash
curl -fsSL https://raw.githubusercontent.com/your-repo/specter/main/install.sh | sh
```

📖 **[Complete installation guide](INSTALL.md)** - Including Docker, Rust setup, troubleshooting, etc.

### Initialize Project

```bash
cd your-project
specter init
```

This creates:
```
.specter/
  ├── config.toml
  └── scripts/
specs/
changes/
```

### Implement AI Scripts

Specter requires you to implement AI integration scripts in `.specter/scripts/`:

1. `gemini-proposal.sh` - Call Gemini CLI to generate proposals
2. `codex-challenge.sh` - Call Codex CLI to challenge proposals
3. `gemini-reproposal.sh` - Call Gemini CLI to refine proposals
4. `claude-implement.sh` - Call Claude Code to implement
5. `codex-verify.sh` - Call Codex CLI to generate and run tests

Example `gemini-proposal.sh`:
```bash
#!/bin/bash
CHANGE_ID="$1"
DESCRIPTION="$2"

# Call gemini-cli with your prompt
gemini /openspec:proposal "$CHANGE_ID" "$DESCRIPTION" \
  --output-format stream-json \
  --context "changes/$CHANGE_ID"
```

## 📖 Workflow

```bash
# 1. Generate proposal (Gemini)
specter proposal add-oauth "Add OAuth authentication"

# 2. Challenge the proposal (Codex)
specter challenge add-oauth

# 3. Refine based on feedback (Gemini, automatic)
specter reproposal add-oauth

# 4. Re-challenge to verify fixes (optional)
specter challenge add-oauth

# 5. Implement the proposal (Claude)
specter implement add-oauth

# 6. Verify with tests (Codex)
specter verify add-oauth

# 7. Archive when complete
specter archive add-oauth
```

## 🎨 Interactive UI Example

```bash
$ specter challenge add-oauth

🔍 Analyzing proposal with Codex...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100% (23s)

📊 Challenge Report Generated

📊 Summary:
   🔴 High:    2 issues
   🟡 Medium:  3 issues
   🟢 Low:     1 issue

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔴 HIGH SEVERITY ISSUE (first)

Architecture Conflict in tasks.md:1.2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⏭️  Next steps:
   1. Review full report:
      cat changes/add-oauth/CHALLENGE.md

   2. Address issues automatically:
      specter reproposal add-oauth

   3. Or edit manually and re-challenge:
      specter challenge add-oauth
```

## 📁 Project Structure

```
project/
├── .specter/
│   ├── config.toml          # Specter configuration
│   └── scripts/             # AI integration scripts
├── specs/                   # Main specifications
│   ├── auth/
│   │   └── spec.md
│   └── api/
│       └── spec.md
└── changes/                 # Change proposals
    ├── add-oauth/
    │   ├── proposal.md      # Gemini generated
    │   ├── tasks.md         # Gemini generated
    │   ├── diagrams.md      # Gemini generated
    │   ├── specs/           # Spec deltas
    │   ├── CHALLENGE.md     # Codex generated
    │   ├── IMPLEMENTATION.md # Claude record
    │   └── VERIFICATION.md  # Codex generated
    └── archive/
```

## 💡 Cost Comparison

| Task | Pure Claude | Specter (Mixed) | Savings |
|------|-------------|-----------------|---------|
| Proposal generation (100+ files) | $$$$ | $ | 80% |
| Code challenge/review | $$$ | $ | 75% |
| Implementation | $$ | $$ | 0% |
| Test generation | $$ | $ | 60% |
| **Total** | **$15-20** | **$4-5** | **70-75%** |

## 🔧 Commands

### Core Commands

- `specter proposal <id> <description>` - Generate proposal with Gemini
- `specter challenge <id>` - Challenge proposal with Codex
- `specter reproposal <id>` - Refine proposal based on challenge
- `specter implement <id>` - Implement with Claude
- `specter verify <id>` - Generate tests and verify with Codex
- `specter archive <id>` - Archive completed change

### Utility Commands

- `specter init` - Initialize Specter in current directory
- `specter list` - List all active changes
- `specter list --archived` - List archived changes
- `specter status <id>` - Show change status
- `specter refine <id> <requirements>` - Manually add requirements

## 🏗️ Architecture

Specter is built in Rust for:
- ⚡ **Performance** - 10-20x faster than Node.js alternatives
- 🔒 **Type Safety** - Compile-time guarantees
- 📦 **Single Binary** - No runtime dependencies
- 🎯 **Reliability** - Robust error handling

## 📚 Documentation

See [design document](/tmp/specter-design.md) for detailed architecture.

## 🤝 Contributing

Contributions welcome! This is an open-source project.

## 📄 License

MIT License

---

**Built for cost-effective, high-quality spec-driven development**

**Key Benefits:**
- 🎯 Iterative proposal refinement through AI challenge
- 💰 70-75% cost reduction vs pure Claude approach
- 🤖 Best tool for each job (Gemini/Codex/Claude orchestration)
- 📋 Automated testing and verification
- 🚀 2M context window for large codebase exploration
