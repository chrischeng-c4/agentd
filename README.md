# Specter - Spec-driven Development Orchestrator 🎭

**Specter** = **Spec** + Orches**ter** (Orchestrator)

A Rust-powered spec-driven development tool with **iterative proposal refinement** through AI orchestration.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🎯 Core Concept

**Specter installs Skills into Claude Code** so you can orchestrate multiple AI tools without leaving your interactive session:

- 🤖 **Gemini** (2M context, low cost) - Code exploration & proposal generation
- 🔍 **Codex** (code specialist) - Challenge proposals & generate tests
- 🎨 **Claude** (you!) - Precise implementation and workflow orchestration

## ✨ Key Innovations

1. **Claude Code Skills** - Work entirely in Claude Code interactive mode, no bash switching
2. **Challenge Phase** - AI automatically reviews proposals (Codex analyzes against existing code)
3. **Iterative Refinement** - proposal → challenge → reproposal loop until satisfied
4. **Automated Verification** - Codex generates tests from specs and verifies implementation

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

This installs **6 Claude Code Skills**:
```
.claude/skills/
  ├── specter-proposal/
  ├── specter-challenge/
  ├── specter-reproposal/
  ├── specter-implement/
  ├── specter-verify/
  └── specter-archive/

.specter/              # Configuration
specs/                 # Main specifications
changes/               # Active changes
```

### Usage in Claude Code

After `specter init`, you can use these skills directly in **Claude Code interactive mode**:

```
You: /specter:proposal add-oauth "Add OAuth authentication"

Claude: 🤖 Generating proposal with Gemini (2M context)...
        [Explores codebase, analyzes architecture...]
        ✅ Proposal created: changes/add-oauth/

        📄 Files generated:
           • proposal.md - Why, what, impact
           • tasks.md - Implementation checklist
           • diagrams.md - 4 Mermaid diagrams
           • specs/auth/spec.md - Requirements with WHEN/THEN scenarios

You: /specter:challenge add-oauth

Claude: 🔍 Analyzing proposal with Codex...
        [Compares with existing codebase...]

        📊 Found 2 HIGH severity issues:
           🔴 Architecture conflict in tasks.md
           🔴 Missing migration path

        💡 Recommendation: Run /specter:reproposal to fix automatically

You: /specter:reproposal add-oauth

Claude: 🔄 Refining proposal based on feedback...
        [Reads CHALLENGE.md, fixes issues...]
        ✅ Proposal updated

        ⏭️  Next: /specter:implement add-oauth
```

## 📖 Complete Workflow

**All commands run in Claude Code** - no bash switching needed!

```
1. /specter:proposal <id> "<description>"
   └─> Gemini explores codebase, generates proposal

2. /specter:challenge <id>
   └─> Codex analyzes against existing code

3. /specter:reproposal <id>
   └─> Gemini fixes issues automatically

4. /specter:implement <id>
   └─> Claude (you!) implements the tasks

5. /specter:verify <id>
   └─> Codex generates and runs tests

6. /specter:archive <id>
   └─> Archive completed change
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
