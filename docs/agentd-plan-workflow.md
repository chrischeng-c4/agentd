# Agentd Plan 完整工作流程

## 概覽

`agentd plan` 是 Spec-Driven Development 的核心命令，負責整個 Planning Phase：
- Clarification（需求釐清）
- Proposal Generation（提案生成）
- Challenge（代碼審查）
- Human-in-the-Loop Decision（人類決策點）

預設使用 **Human-in-the-Loop (HITL) Mode**。

---

## 完整流程圖

```
┌─────────────────────────────────────────────────────────────────┐
│ User: /agentd:plan <change-id> "<description>"                  │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Skill: agentd:plan (Claude Code Main Thread)                    │
│ - Parse arguments: change_id, description                       │
│ - Check if change exists: STATE.yaml in agentd/changes/         │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
              ┌────────┴────────┐
              │                 │
         [New Change]      [Existing Change]
              │                 │
              ↓                 ↓
┌──────────────────────┐  ┌─────────────────────────┐
│ Check STATE.yaml     │  │ Read STATE.yaml         │
│ → Does not exist     │  │ → Check phase field     │
└──────┬───────────────┘  └─────────┬───────────────┘
       │                            │
       ↓                            ↓
┌──────────────────────────────────┴────────────────────────┐
│ Phase Decision                                            │
│ - No STATE.yaml → Clarification Phase                     │
│ - proposed → Continue Planning                            │
│ - challenged → Planning complete, suggest /agentd:impl    │
│ - rejected → Show error, suggest manual review            │
└──────────────────────┬────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ ═══════════════════════════════════════════════════════════════ │
│ PHASE 1: CLARIFICATION (New Changes Only)                       │
│ ═══════════════════════════════════════════════════════════════ │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Skill: Analyze description for ambiguities                      │
│ - Check if description is detailed enough                       │
│ - Identify key decision points (auth method, DB, API, etc.)     │
│ - Prepare 3-5 clarification questions                           │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
         ┌─────────────┴─────────────┐
         │                           │
    [Skip Clarify?]            [Need Clarify]
         │                           │
         ↓                           ↓
    Skip if:                   ┌─────────────────────────────────┐
    - --skip-clarify flag      │ AskUserQuestion Tool (Q1)       │
    - Description detailed     │ ─────────────────────────────── │
                               │ question: "Auth method?"        │
                               │ header: "Auth Method"           │
                               │ options:                        │
                               │   - OAuth (Recommended)         │
                               │   - JWT                         │
                               │   - Session-based               │
                               │ multiSelect: false              │
                               └─────────────┬───────────────────┘
                                             ↓
                               ┌─────────────────────────────────┐
                               │ AskUserQuestion Tool (Q2)       │
                               │ ─────────────────────────────── │
                               │ question: "Database?"           │
                               │ header: "Database"              │
                               │ options: [PostgreSQL, MySQL...] │
                               └─────────────┬───────────────────┘
                                             ↓
                               ┌─────────────────────────────────┐
                               │ AskUserQuestion Tool (Q3-Q5)    │
                               │ ... (max 5 questions total)     │
                               └─────────────┬───────────────────┘
                                             ↓
                               ┌─────────────────────────────────┐
                               │ User Answers Received           │
                               │ - Store answers in memory       │
                               └─────────────┬───────────────────┘
                                             ↓
                               ┌─────────────────────────────────┐
                               │ MCP Tool: create_clarifications │
                               │ ─────────────────────────────── │
                               │ {                               │
                               │   change_id: "add-oauth",       │
                               │   questions: [                  │
                               │     {                           │
                               │       topic: "Auth Method",     │
                               │       question: "...",          │
                               │       answer: "OAuth",          │
                               │       rationale: "..."          │
                               │     }                           │
                               │   ]                             │
                               │ }                               │
                               └─────────────┬───────────────────┘
                                             ↓
                               ┌─────────────────────────────────┐
                               │ MCP Tool Execution              │
                               │ ─────────────────────────────── │
                               │ 1. Create directory if needed:  │
                               │    agentd/changes/<change-id>/  │
                               │ 2. Generate frontmatter:        │
                               │    ---                          │
                               │    change: <change-id>          │
                               │    date: 2026-01-19             │
                               │    ---                          │
                               │ 3. Write Q&A sections           │
                               │ 4. Save to clarifications.md    │
                               │ ✅ Return success message       │
                               └─────────────┬───────────────────┘
                                             ↓
┌─────────────────────────────────────────────────────────────────┐
│ Git Workflow Selection (New Changes Only)                       │
│ ─────────────────────────────────────────────────────────────── │
│ AskUserQuestion:                                                │
│   question: "Preferred Git workflow?"                           │
│   header: "Git Setup"                                           │
│   options:                                                      │
│     - "New branch (Recommended)"                                │
│       → git checkout -b agentd/<change-id>                      │
│     - "New worktree"                                            │
│       → git worktree add -b agentd/<change-id> ../path          │
│     - "In place"                                                │
│       → Stay on current branch                                  │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
         ┌─────────────┴─────────────┐
         │                           │
    [New Branch]              [New Worktree]        [In Place]
         │                           │                    │
         ↓                           ↓                    ↓
    git checkout -b          git worktree add         (No action)
    agentd/<change-id>       -b agentd/<change-id>
                             ../<project>-agentd/
                                   <change-id>
         │                           │                    │
         └───────────────────────────┴────────────────────┘
                                     ↓
┌─────────────────────────────────────────────────────────────────┐
│ ═══════════════════════════════════════════════════════════════ │
│ PHASE 2: PROPOSAL GENERATION                                    │
│ ═══════════════════════════════════════════════════════════════ │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Skill: Execute agentd plan command                              │
│ ─────────────────────────────────────────────────────────────── │
│ Bash: agentd plan <change-id> "<description>"                   │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ CLI: src/main.rs - Commands::Plan                               │
│ ─────────────────────────────────────────────────────────────── │
│ 1. Load AgentdConfig from agentd/config.toml                    │
│ 2. Delegate to: agentd::cli::plan::run()                        │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ src/cli/plan.rs::run()                                          │
│ ─────────────────────────────────────────────────────────────── │
│ 1. Check if new or existing change (STATE.yaml exists?)         │
│ 2. For new changes: require description parameter               │
│ 3. Check clarifications.md (unless --skip-clarify):             │
│    - Path: agentd/changes/<change-id>/clarifications.md         │
│    - If missing → Error with helpful message                    │
│ 4. Create ProposalEngineConfig                                  │
│ 5. Call: proposal_engine::run_proposal_loop(config)             │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ src/cli/proposal_engine.rs::run_proposal_loop()                 │
│ ─────────────────────────────────────────────────────────────── │
│ 1. Read config.workflow.human_in_loop                           │
│    → Default: true (HITL mode)                                  │
│ 2. Branch based on mode                                         │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
              ┌────────┴────────┐
              │                 │
      [human_in_loop = true]  [human_in_loop = false]
              │                 │
              ↓                 ↓
   ┌─────────────────┐    ┌──────────────────┐
   │ HITL MODE       │    │ AUTOMATED MODE   │
   │ (DEFAULT)       │    │ (Legacy)         │
   └────────┬────────┘    └────────┬─────────┘
            │                      │
            │                      └─────────────────┐
            │                                        │
            ↓                                        ↓
┌─────────────────────────────────────────┐  ┌───────────────────┐
│ ═══════════════════════════════════════ │  │ AUTOMATED MODE:   │
│ HITL MODE DETAILED FLOW                 │  │ See Appendix A    │
│ ═══════════════════════════════════════ │  └───────────────────┘
└──────────────────┬──────────────────────┘
                   ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Sequential Generation - run_proposal_step_sequential()  │
│ ═══════════════════════════════════════════════════════════════ │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 1.1: Generate proposal.md                                 │
│ ─────────────────────────────────────────────────────────────── │
│ 1. Create change directory: agentd/changes/<resolved-id>/       │
│ 2. Resolve change-id conflicts (add -1, -2 if exists)           │
│ 3. Generate GEMINI.md context:                                  │
│    - crate::context::generate_gemini_context()                  │
│    - Phase: ContextPhase::Proposal                              │
│    - Includes: CLAUDE.md, clarifications.md, project structure  │
│ 4. Assess complexity: change.assess_complexity(project_root)    │
│    → Returns: Low / Medium / High / Critical                    │
│ 5. Select Gemini model based on complexity:                     │
│    - Medium → gemini-3-flash-preview (default)                  │
│    - Critical → gemini-3-pro-preview                            │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ 6. Build prompt: prompts::gemini_proposal_with_mcp_prompt()     │
│ ─────────────────────────────────────────────────────────────── │
│ Prompt content:                                                 │
│ """                                                             │
│ You are tasked with creating a proposal for change: <id>       │
│                                                                 │
│ Description: <description>                                      │
│                                                                 │
│ Use the create_proposal MCP tool to generate:                  │
│ - Summary: High-level overview                                 │
│ - Why: Business justification                                  │
│ - What changes: List of modifications                          │
│ - Impact:                                                       │
│   - Scope: patch/minor/major                                   │
│   - Affected specs: List spec IDs (e.g., `auth-flow`, ...)     │
│   - Affected files: Estimate count                             │
│                                                                 │
│ Read GEMINI.md and clarifications.md for context.              │
│ """                                                             │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ 7. Execute: GeminiOrchestrator::run_one_shot()                  │
│ ─────────────────────────────────────────────────────────────── │
│ Function: src/orchestrator/gemini.rs::run_one_shot()            │
│ Parameters:                                                     │
│   - change_id: <resolved-id>                                    │
│   - prompt: (from step 6)                                       │
│   - complexity: Medium                                          │
│                                                                 │
│ Implementation:                                                 │
│   1. Ensure Gemini MCP config exists:                           │
│      - crate::mcp::ensure_gemini_mcp_config()                   │
│      - Checks: .gemini/mcp.json                                 │
│   2. Build environment variables:                               │
│      - GEMINI_CONTEXT_FILE=agentd/changes/<id>/GEMINI.md        │
│   3. Build CLI args:                                            │
│      - Command: "agentd:one-shot"                               │
│      - Model: gemini-3-flash-preview                            │
│      - NO --resume flag (fresh session)                         │
│   4. Execute: runner.run_llm_with_cwd()                         │
│      - Provider: LlmProvider::Gemini                            │
│      - CWD: project_root                                        │
│      - Capture output: true                                     │
│                                                                 │
│ Gemini CLI execution:                                           │
│   → gemini chat --model gemini-3-flash-preview \                │
│      --mcp .gemini/mcp.json \                                   │
│      --prompt "<prompt>"                                        │
│                                                                 │
│ Gemini's actions (autonomous):                                  │
│   1. Read GEMINI.md via MCP read_file tool                      │
│   2. Read clarifications.md via MCP read_file tool              │
│   3. Analyze requirements                                       │
│   4. Call MCP create_proposal tool:                             │
│      {                                                          │
│        change_id: "add-oauth",                                  │
│        summary: "Add OAuth authentication",                     │
│        why: "Enable users to login with Google/GitHub",         │
│        what_changes: [                                          │
│          "Add OAuth provider integration",                      │
│          "Create user session management",                      │
│          "Add OAuth callback endpoints"                         │
│        ],                                                       │
│        impact: {                                                │
│          scope: "minor",                                        │
│          affected_specs: ["auth-flow", "user-model",            │
│                           "api-endpoints"],                     │
│          affected_files: 8,                                     │
│          affected_code: ["src/auth/", "src/models/"],           │
│          breaking_changes: null                                 │
│        }                                                        │
│      }                                                          │
│   5. MCP server executes create_proposal:                       │
│      - Validates input schema                                   │
│      - Generates frontmatter with checksum                      │
│      - Writes to: agentd/changes/<id>/proposal.md               │
│   6. Return success message                                     │
│                                                                 │
│ Return values:                                                  │
│   - output: String (Gemini's response text)                     │
│   - usage: UsageMetrics {                                       │
│       tokens_in: 15234,                                         │
│       tokens_out: 892,                                          │
│       duration_ms: 4521,                                        │
│       session_id: None (one-shot, no session tracking)          │
│     }                                                           │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ 8. Record usage to STATE.yaml                                   │
│ ─────────────────────────────────────────────────────────────── │
│ Function: record_usage()                                        │
│ - Step: "proposal-gen"                                          │
│ - Model: "gemini-3-flash-preview"                               │
│ - Tokens in: 15234, out: 892                                    │
│ - Cost calculation:                                             │
│   - Input: 15234 / 1M * $0.1 = $0.0015                          │
│   - Output: 892 / 1M * $0.4 = $0.0004                           │
│   - Total: $0.0019                                              │
│ - Write to: agentd/changes/<id>/STATE.yaml                      │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 1.2: Self-Review Loop (Max 3 iterations)                  │
│ ─────────────────────────────────────────────────────────────── │
│ For iteration 0 to 2:                                           │
│   1. Build review prompt:                                       │
│      prompts::proposal_self_review_with_mcp_prompt(change_id)   │
│                                                                 │
│      Prompt content:                                            │
│      """                                                        │
│      Review the proposal.md file for change: <id>               │
│                                                                 │
│      Use MCP read_file to read:                                 │
│      - agentd/changes/<id>/proposal.md                          │
│                                                                 │
│      Check for:                                                 │
│      1. Completeness (all required sections)                    │
│      2. Clarity (clear, specific descriptions)                  │
│      3. Affected specs format (proper markdown list)            │
│      4. Impact assessment accuracy                              │
│                                                                 │
│      If issues found:                                           │
│      - Use MCP edit_file to fix them                            │
│      - Output: "NEEDS_REVISION: <issues>"                       │
│                                                                 │
│      If no issues:                                              │
│      - Output: "PASS: No changes needed"                        │
│      """                                                        │
│                                                                 │
│   2. Execute: orchestrator.run_one_shot() (FRESH SESSION)       │
│      - Same as step 7, but with review prompt                   │
│      - Gemini reads proposal.md via MCP                         │
│      - Gemini analyzes quality                                  │
│      - If issues: Gemini edits via MCP edit_file                │
│                                                                 │
│   3. Detect review result: detect_self_review_marker(output)    │
│      - Parse Gemini's output for markers:                       │
│        - "PASS" → SelfReviewResult::Pass                        │
│        - "NEEDS_REVISION" → SelfReviewResult::NeedsRevision     │
│                                                                 │
│   4. Record usage: "proposal-review"                            │
│                                                                 │
│   5. Handle result:                                             │
│      - PASS → Break loop, continue to next phase                │
│      - NEEDS_REVISION → Continue to next iteration              │
│                                                                 │
│   6. If iteration 2 (last) and still NEEDS_REVISION:            │
│      - Print warning: "Max review iterations reached"           │
│      - Continue anyway (don't block)                            │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 1.3: Parse affected_specs from proposal.md                │
│ ─────────────────────────────────────────────────────────────── │
│ Function: parse_affected_specs(proposal_content)                │
│ File: src/parser/proposal.rs                                    │
│                                                                 │
│ 1. Read proposal.md content                                     │
│ 2. Search for regex pattern:                                    │
│    r"(?mi)^[-*]\s*Affected specs:\s*(.+?)$"                     │
│                                                                 │
│ 3. Parse formats:                                               │
│    - Backticks: `auth-flow`, `user-model`                       │
│    - Array: ["auth-flow", "user-model"]                         │
│    - Plain: auth-flow, user-model                               │
│                                                                 │
│ 4. Clean and split:                                             │
│    - Remove: [], `, ", '                                        │
│    - Split by comma                                             │
│    - Trim whitespace                                            │
│    - Filter empty/none/n/a                                      │
│                                                                 │
│ 5. Return: Vec<String>                                          │
│    → ["auth-flow", "user-model", "api-endpoints"]               │
│                                                                 │
│ 6. If empty:                                                    │
│    → Print: "No specs required for this change"                 │
│    → Skip Phase 2                                               │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 2: Generate Specs Sequentially                            │
│ ─────────────────────────────────────────────────────────────── │
│ For each spec_id in affected_specs (in order):                  │
│   Print: "Spec 1/3: auth-flow"                                  │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 2.1: Generate spec (auth-flow)                            │
│ ─────────────────────────────────────────────────────────────── │
│ 1. Prepare context_specs (previously generated specs):          │
│    - For spec #1: context_specs = []                            │
│    - For spec #2: context_specs = ["auth-flow"]                 │
│    - For spec #3: context_specs = ["auth-flow", "user-model"]   │
│                                                                 │
│ 2. Build prompt: prompts::gemini_spec_with_mcp_prompt()         │
│    Parameters:                                                  │
│      - change_id: "add-oauth"                                   │
│      - spec_id: "auth-flow"                                     │
│      - other_specs: context_specs                               │
│                                                                 │
│    Prompt content:                                              │
│    """                                                          │
│    Create spec for: auth-flow (change: add-oauth)               │
│                                                                 │
│    First, read context via MCP read_file:                       │
│    1. agentd/changes/add-oauth/proposal.md                      │
│    2. agentd/changes/add-oauth/clarifications.md                │
│    [If context_specs not empty:]                                │
│    3. agentd/changes/add-oauth/specs/auth-flow.md               │
│       (previously generated specs for reference)                │
│                                                                 │
│    Then use MCP create_spec tool:                               │
│    {                                                            │
│      change_id: "add-oauth",                                    │
│      spec_id: "auth-flow",                                      │
│      title: "OAuth Authentication Flow",                        │
│      overview: "Defines the OAuth authentication flow...",      │
│      requirements: [                                            │
│        {                                                        │
│          id: "R1",                                              │
│          title: "OAuth provider integration",                   │
│          description: "...",                                    │
│          priority: "high"                                       │
│        }                                                        │
│      ],                                                         │
│      scenarios: [                                               │
│        {                                                        │
│          name: "User logs in with Google",                      │
│          given: "User is not authenticated",                    │
│          when: "User clicks 'Login with Google'",               │
│          then: "OAuth flow starts and user is redirected"       │
│        }                                                        │
│      ],                                                         │
│      flow_diagram: "```mermaid\n...\n```" (optional)            │
│    }                                                            │
│    """                                                          │
│                                                                 │
│ 3. Execute: orchestrator.run_one_shot() (FRESH SESSION)         │
│    - Gemini reads proposal.md, clarifications.md, prev specs    │
│    - Gemini analyzes requirements                               │
│    - Gemini calls MCP create_spec                               │
│    - MCP server validates and writes:                           │
│      agentd/changes/add-oauth/specs/auth-flow.md                │
│                                                                 │
│ 4. Record usage: "spec-gen-auth-flow"                           │
│                                                                 │
│ 5. Print: "✅ auth-flow.md generated"                           │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 2.2: Self-Review Loop for spec (Max 3 iterations)         │
│ ─────────────────────────────────────────────────────────────── │
│ For iteration 0 to 2:                                           │
│   1. Build review prompt:                                       │
│      prompts::spec_self_review_with_mcp_prompt()                │
│      Parameters:                                                │
│        - change_id: "add-oauth"                                 │
│        - spec_id: "auth-flow"                                   │
│        - other_specs: context_specs                             │
│                                                                 │
│      Prompt content:                                            │
│      """                                                        │
│      Review spec: agentd/changes/add-oauth/specs/auth-flow.md   │
│                                                                 │
│      Read via MCP read_file:                                    │
│      1. specs/auth-flow.md (the spec to review)                 │
│      2. proposal.md (for consistency check)                     │
│      [If other_specs:]                                          │
│      3. Other specs (for cross-spec consistency)                │
│                                                                 │
│      Check for:                                                 │
│      1. Requirements completeness (covers all aspects)          │
│      2. Scenarios quality (proper Given/When/Then)              │
│      3. Consistency with proposal                               │
│      4. No contradictions with other specs                      │
│                                                                 │
│      If issues: Edit via MCP edit_file and output NEEDS_REVISION│
│      If good: Output PASS                                       │
│      """                                                        │
│                                                                 │
│   2. Execute: orchestrator.run_one_shot() (FRESH SESSION)       │
│   3. Detect result: PASS or NEEDS_REVISION                      │
│   4. Record usage: "spec-review-auth-flow"                      │
│   5. Handle result (same as proposal review)                    │
│                                                                 │
│   Print: "✓ Review 1: PASS" or "⚠ Review 1: NEEDS_REVISION"    │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
                [Repeat Phase 2.1-2.2 for each spec]
                       ↓
              spec #2: user-model
                       ↓
              spec #3: api-endpoints
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 3: Generate tasks.md                                      │
│ ─────────────────────────────────────────────────────────────── │
│ 1. Prepare all_files list for context:                          │
│    all_files = [                                                │
│      "proposal.md",                                             │
│      "specs/auth-flow.md",                                      │
│      "specs/user-model.md",                                     │
│      "specs/api-endpoints.md"                                   │
│    ]                                                            │
│                                                                 │
│ 2. Build prompt: prompts::gemini_tasks_with_mcp_prompt()        │
│    Parameters:                                                  │
│      - change_id: "add-oauth"                                   │
│      - all_spec_ids: ["auth-flow", "user-model", ...]           │
│                                                                 │
│    Prompt content:                                              │
│    """                                                          │
│    Create tasks.md for change: add-oauth                        │
│                                                                 │
│    Read all context files via MCP read_file:                    │
│    1. agentd/changes/add-oauth/proposal.md                      │
│    2. agentd/changes/add-oauth/specs/auth-flow.md               │
│    3. agentd/changes/add-oauth/specs/user-model.md              │
│    4. agentd/changes/add-oauth/specs/api-endpoints.md           │
│                                                                 │
│    Then use MCP create_tasks tool:                              │
│    {                                                            │
│      change_id: "add-oauth",                                    │
│      tasks: [                                                   │
│        {                                                        │
│          layer: "data",                                         │
│          number: 1,                                             │
│          title: "Create User model",                            │
│          file: {                                                │
│            path: "src/models/user.rs",                          │
│            action: "CREATE"                                     │
│          },                                                     │
│          spec_ref: "user-model:R1",                             │
│          description: "Define User struct with OAuth fields",   │
│          depends: []                                            │
│        },                                                       │
│        {                                                        │
│          layer: "logic",                                        │
│          number: 1,                                             │
│          title: "Implement OAuth provider",                     │
│          file: {                                                │
│            path: "src/auth/oauth.rs",                           │
│            action: "CREATE"                                     │
│          },                                                     │
│          spec_ref: "auth-flow:R1",                              │
│          description: "...",                                    │
│          depends: ["data.1"]                                    │
│        }                                                        │
│      ]                                                          │
│    }                                                            │
│    """                                                          │
│                                                                 │
│ 3. Execute: orchestrator.run_one_shot() (FRESH SESSION)         │
│    - Gemini reads all files                                     │
│    - Gemini creates task breakdown                              │
│    - Gemini calls MCP create_tasks                              │
│    - MCP server writes: agentd/changes/add-oauth/tasks.md       │
│                                                                 │
│ 4. Record usage: "tasks-gen"                                    │
│                                                                 │
│ 5. Print: "✅ tasks.md generated"                               │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 3.2: Self-Review Loop for tasks (Max 3 iterations)        │
│ ─────────────────────────────────────────────────────────────── │
│ For iteration 0 to 2:                                           │
│   1. Build review prompt:                                       │
│      prompts::tasks_self_review_with_mcp_prompt()               │
│      Parameters:                                                │
│        - change_id: "add-oauth"                                 │
│        - all_files: ["proposal.md", "specs/..."]                │
│                                                                 │
│      Prompt content:                                            │
│      """                                                        │
│      Review tasks.md for change: add-oauth                      │
│                                                                 │
│      Read via MCP read_file:                                    │
│      1. tasks.md (the file to review)                           │
│      2. proposal.md                                             │
│      3. All specs                                               │
│                                                                 │
│      Check for:                                                 │
│      1. All requirements covered (cross-ref with specs)         │
│      2. Proper task layering (data → logic → integration)       │
│      3. Correct dependencies (no circular deps)                 │
│      4. File paths are reasonable                               │
│      5. Spec references are valid                               │
│                                                                 │
│      If issues: Edit via MCP edit_file and output NEEDS_REVISION│
│      If good: Output PASS                                       │
│      """                                                        │
│                                                                 │
│   2. Execute: orchestrator.run_one_shot() (FRESH SESSION)       │
│   3. Detect result: PASS or NEEDS_REVISION                      │
│   4. Record usage: "tasks-review"                               │
│   5. Handle result                                              │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: Validate Proposal Format (Local, No AI)                 │
│ ─────────────────────────────────────────────────────────────── │
│ Function: run_validate_proposal_step()                          │
│ File: src/cli/validate_proposal.rs                              │
│                                                                 │
│ 1. Load validation config from agentd/config.toml:              │
│    [validation]                                                 │
│    required_headings = ["Overview", "Acceptance Criteria"]      │
│    scenario_pattern = 'WHEN\s.*THEN\s'                          │
│    scenario_min_count = 1                                       │
│                                                                 │
│ 2. Parse proposal.md:                                           │
│    - Check frontmatter exists and valid                         │
│    - Check all required headings present                        │
│    - Validate requirement format (R1, R2, ...)                  │
│    - Validate scenario format (WHEN...THEN...)                  │
│    - Check for broken references                                │
│                                                                 │
│ 3. Parse all spec files:                                        │
│    - Same validations for each spec                             │
│                                                                 │
│ 4. Validate tasks.md:                                           │
│    - Check task YAML blocks are valid                           │
│    - Check spec_ref references exist                            │
│    - Check file paths are not absolute                          │
│                                                                 │
│ 5. Return ValidationSummary:                                    │
│    {                                                            │
│      high_count: 0,                                             │
│      medium_count: 1,                                           │
│      low_count: 3,                                              │
│      errors: [...]                                              │
│    }                                                            │
│                                                                 │
│ 6. Print result:                                                │
│    - If valid: "✅ Proposal format validation passed"           │
│    - If invalid: "⚠️ Format validation failed"                  │
│                                                                 │
│ 7. For HITL mode:                                               │
│    - If invalid → Exit with error (no auto-fix)                 │
│    - User must manually fix                                     │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 3: Challenge with Codex                                    │
│ ─────────────────────────────────────────────────────────────── │
│ Function: run_challenge_step()                                  │
│                                                                 │
│ 1. Validate structure:                                          │
│    - Change::new().validate_structure()                         │
│    - Check all files exist                                      │
│                                                                 │
│ 2. Generate AGENTS.md context:                                  │
│    - crate::context::generate_agents_context()                  │
│    - Phase: ContextPhase::Challenge                             │
│    - Includes: CLAUDE.md, proposal.md, all specs, tasks.md      │
│                                                                 │
│ 3. Create CHALLENGE.md skeleton:                                │
│    - crate::context::create_challenge_skeleton()                │
│    - Template with sections: Verdict, Issues, Summary           │
│                                                                 │
│ 4. Assess complexity (same as proposal)                         │
│                                                                 │
│ 5. Select Codex model:                                          │
│    - Medium complexity → gpt-5.2-codex (reasoning: medium)      │
│    - Critical complexity → gpt-5.2-codex (reasoning: extra high)│
│                                                                 │
│ 6. Build Codex environment:                                     │
│    - AGENTS_CONTEXT_FILE=agentd/changes/<id>/AGENTS.md          │
│                                                                 │
│ 7. Execute CodexOrchestrator::run_challenge():                  │
│    - Command: codex chat --model gpt-5.2-codex \                │
│               --reasoning medium \                              │
│               --prompt "<challenge_prompt>"                     │
│                                                                 │
│    Challenge prompt (built-in):                                 │
│    """                                                          │
│    You are a senior code reviewer. Review this proposal:        │
│                                                                 │
│    Read the proposal, specs, and tasks from AGENTS.md.          │
│                                                                 │
│    Evaluate:                                                    │
│    1. Technical feasibility                                     │
│    2. Architecture soundness                                    │
│    3. Security concerns                                         │
│    4. Performance implications                                  │
│    5. Completeness of requirements                              │
│    6. Task breakdown quality                                    │
│                                                                 │
│    For each issue found, specify:                               │
│    - **Severity**: High / Medium / Low                          │
│    - **Description**: What's wrong                              │
│    - **Suggestion**: How to fix                                 │
│    - **Spec Reference**: Which spec/requirement                 │
│                                                                 │
│    Final verdict:                                               │
│    - APPROVED: Ready for implementation                         │
│    - NEEDS_REVISION: Fixable issues found                       │
│    - REJECTED: Fundamental flaws                                │
│                                                                 │
│    Write full review to CHALLENGE.md.                           │
│    """                                                          │
│                                                                 │
│ 8. Codex execution (autonomous):                                │
│    - Reads AGENTS.md (includes all proposal files)              │
│    - Analyzes code patterns, architecture                       │
│    - Identifies issues                                          │
│    - Writes CHALLENGE.md with detailed review                   │
│                                                                 │
│ 9. Record usage: "challenge"                                    │
│    - Model: gpt-5.2-codex                                       │
│    - Tokens: ~20K input, ~2K output                             │
│    - Cost: ~$0.06                                               │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 4: Parse Challenge Verdict                                 │
│ ─────────────────────────────────────────────────────────────── │
│ Function: parse_challenge_verdict()                             │
│ File: src/parser/challenge.rs                                   │
│                                                                 │
│ 1. Read CHALLENGE.md                                            │
│ 2. Search for verdict markers:                                  │
│    - "**Verdict**: APPROVED" → ChallengeVerdict::Approved       │
│    - "**Verdict**: NEEDS_REVISION" → NeedsRevision              │
│    - "**Verdict**: REJECTED" → Rejected                         │
│    - Otherwise → Unknown                                        │
│                                                                 │
│ 3. Count severity levels:                                       │
│    - high_count = content.matches("**Severity**: High").count() │
│    - medium_count = matches("**Severity**: Medium")             │
│    - low_count = matches("**Severity**: Low")                   │
│                                                                 │
│ 4. Display summary:                                             │
│    - APPROVED: "✅ APPROVED - Ready for implementation!"        │
│    - NEEDS_REVISION: "⚠️ NEEDS_REVISION - Found 2 HIGH,         │
│                       3 MEDIUM, 1 LOW severity issues"          │
│    - REJECTED: "❌ REJECTED - Fundamental problems"             │
│                                                                 │
│ 5. Update STATE.yaml:                                           │
│    - phase: "challenged" (if APPROVED)                          │
│    - phase: "proposed" (if NEEDS_REVISION)                      │
│    - phase: "rejected" (if REJECTED)                            │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ ═══════════════════════════════════════════════════════════════ │
│ HITL MODE: STOP HERE AND RETURN TO SKILL                        │
│ ═══════════════════════════════════════════════════════════════ │
│                                                                 │
│ Return: ProposalEngineResult {                                  │
│   resolved_change_id: "add-oauth",                              │
│   verdict: ChallengeVerdict::NeedsRevision,                     │
│   iteration_count: 0                                            │
│ }                                                               │
│                                                                 │
│ Print message:                                                  │
│ "For HITL mode, we stop after first challenge                   │
│  Skill will use AskUserQuestion to let user decide next action" │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ ═══════════════════════════════════════════════════════════════ │
│ PHASE 3: HUMAN-IN-THE-LOOP DECISION POINT                       │
│ ═══════════════════════════════════════════════════════════════ │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────────────────┐
│ Skill: Check agentd plan exit code and STATE.yaml               │
│ ─────────────────────────────────────────────────────────────── │
│ 1. Read STATE.yaml:                                             │
│    phase: "challenged" or "proposed"                            │
│                                                                 │
│ 2. Read CHALLENGE.md and parse verdict                          │
└──────────────────────┬──────────────────────────────────────────┘
                       ↓
              ┌────────┴────────┐
              │                 │
         [APPROVED]      [NEEDS_REVISION]      [REJECTED]
              │                 │                    │
              ↓                 ↓                    ↓
┌──────────────────┐  ┌─────────────────────┐  ┌──────────────────┐
│ Print success    │  │ AskUserQuestion     │  │ Print rejection  │
│ message          │  │ ─────────────────── │  │ message          │
│                  │  │ question: "Proposal │  │                  │
│ Suggest:         │  │  needs revision.    │  │ Read CHALLENGE.md│
│ /agentd:impl     │  │  What to do?"       │  │ Show key issues  │
│ <change-id>      │  │                     │  │                  │
│                  │  │ header: "Next"      │  │ Suggest manual   │
└──────────────────┘  │                     │  │ review and fixes │
                      │ options:            │  └──────────────────┘
                      │   1. "Auto-fix and  │
                      │       rechallenge   │
                      │       (Recommended)"│
                      │   2. "Review        │
                      │       manually"     │
                      │   3. "Stop here"    │
                      └──────────┬──────────┘
                                 ↓
                        [User selects option]
                                 ↓
              ┌──────────────────┴──────────────────┐
              │                  │                  │
         [Auto-fix]       [Review manually]    [Stop here]
              │                  │                  │
              ↓                  ↓                  ↓
┌──────────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│ Execute reproposal   │  │ Read CHALLENGE.md│  │ Print message:   │
│ ──────────────────── │  │ ──────────────── │  │ "You can continue│
│ Bash:                │  │ 1. Parse issues  │  │  manually later  │
│ agentd reproposal \  │  │ 2. Show to user: │  │  with:           │
│   <change-id>        │  │    - Issue #1:   │  │  agentd reproposal│
│                      │  │      Severity: H │  │  <change-id>"    │
│ ↓                    │  │      Desc: ...   │  │                  │
│ Wait for completion  │  │    - Issue #2:   │  │ Exit gracefully  │
│                      │  │      ...         │  └──────────────────┘
│ ↓                    │  │                  │
│ Execute rechallenge  │  │ 3. Ask: "Fix    │
│ ──────────────────── │  │    manually or   │
│ Bash:                │  │    let me help?" │
│ agentd challenge \   │  │                  │
│   <change-id>        │  │ [User decides]   │
│                      │  └──────────────────┘
│ ↓                    │
│ Parse new verdict    │
│                      │
│ ↓                    │
│ LOOP BACK TO         │
│ DECISION POINT       │
│ (max 3-5 iterations) │
└──────────────────────┘

If still NEEDS_REVISION after N iterations:
  → Ask user if they want to continue or stop

If APPROVED:
  → Proceed to /agentd:impl

If REJECTED:
  → Suggest manual review and fixes

---

## Appendix A: Automated Mode (human_in_loop = false)

### Overview

Automated Mode 使用舊的 one-shot generation + auto-reproposal loop 流程。

### 差異對比

| Step | HITL Mode | Automated Mode |
|------|-----------|----------------|
| Generation | Sequential (proposal → specs → tasks) | One-shot (all files in one session) |
| Session | Fresh per phase | Reuse same session |
| Context | File-based via MCP | Session history + files |
| Self-review | Per phase (3 loops each) | Only proposal (3 loops total) |
| Format validation | 1 time, exit if fail | Auto-fix loop (max 2 iterations) |
| Challenge | 1 time, stop | 1 time, then auto-reproposal loop |
| Reproposal | Manual trigger | Automatic on NEEDS_REVISION |
| Iteration limit | None (user controlled) | Max 2 auto-iterations |

### Automated Mode Flow

```
1. run_proposal_step() [Legacy]
   ├─ Create proposal skeleton (ALL files in one go)
   ├─ Run Gemini with ProposalPhase::All
   │  → Generates: proposal.md, specs/*.md, tasks.md
   ├─ Self-review loop (max 3 iterations, SAME session)
   │  → Use --resume latest to continue session
   └─ Save session_id to STATE.yaml

2. Format Validation Loop (max 2 iterations)
   ├─ Validate proposal format
   ├─ If invalid:
   │  ├─ run_reproposal_step() [Auto-fix]
   │  │  ├─ Load session_id from STATE.yaml
   │  │  ├─ Find session index: gemini sessions list
   │  │  ├─ Resume session: --resume <index>
   │  │  └─ Gemini fixes issues in SAME session
   │  └─ Re-validate
   └─ If still invalid after 2 iterations: EXIT

3. run_challenge_step() [Same as HITL]
   └─ Codex reviews and writes CHALLENGE.md

4. Planning Iteration Loop (max 2 iterations)
   ├─ Parse verdict
   ├─ If APPROVED: ✅ Done, phase → challenged
   ├─ If NEEDS_REVISION:
   │  ├─ iteration++
   │  ├─ If iteration > max (2): EXIT with error
   │  ├─ run_reproposal_step() [Auto-fix]
   │  │  ├─ Resume Gemini session (--resume <index>)
   │  │  ├─ Read CHALLENGE.md feedback
   │  │  ├─ Fix issues in SAME session
   │  │  └─ Update all files
   │  ├─ run_rechallenge_step()
   │  │  ├─ Resume Codex session (session reuse)
   │  │  ├─ Re-review updated proposal
   │  │  └─ Update CHALLENGE.md
   │  └─ Loop back to step 4
   └─ If REJECTED: ❌ EXIT with error
```

### Session Reuse Details

**Gemini Session Tracking:**
```rust
// First proposal
let (output, usage) = orchestrator.run_proposal(...).await?;
let session_id = usage.session_id; // UUID

// Save to STATE.yaml
manager.set_session_id(session_id);

// Later reproposal
let session_id = manager.session_id().unwrap();
let session_index = find_session_index(session_id, project_root).await?;

// Resume by index
orchestrator.run_reproposal_with_session(change_id, session_index, ...)
  → gemini chat --resume <index> --prompt "Fix issues..."
```

**Benefits of Session Reuse:**
- Context already loaded (no re-reading)
- Gemini remembers previous generation decisions
- Faster iteration (less token input)

**Drawbacks:**
- Context pollution (accumulated history)
- Confirmation bias (same session reviewing own work)
- Less reproducible (session-dependent)

### When to Use Automated Mode

Use `human_in_loop = false` when:
- ✅ Fully automated CI/CD pipeline
- ✅ Trust AI to auto-fix issues
- ✅ Speed > Quality control
- ✅ Cost of human intervention > token cost

Use `human_in_loop = true` (default) when:
- ✅ Human quality gate needed
- ✅ Cost control important
- ✅ Learning from AI process
- ✅ Fine-grained control desired

---

## Appendix B: File Structures

### Directory Layout After Completion

```
agentd/changes/add-oauth/
├── STATE.yaml              # Change state and LLM usage tracking
├── clarifications.md       # User Q&A from AskUserQuestion
├── proposal.md             # Main proposal document
├── specs/
│   ├── auth-flow.md        # Spec #1 (generated sequentially)
│   ├── user-model.md       # Spec #2 (with context from spec #1)
│   └── api-endpoints.md    # Spec #3 (with context from spec #1-2)
├── tasks.md                # Implementation task breakdown
├── CHALLENGE.md            # Codex code review
├── GEMINI.md               # Context for Gemini (auto-generated)
└── AGENTS.md               # Context for Codex (auto-generated)
```

### STATE.yaml Example

```yaml
change_id: add-oauth
phase: challenged
created_at: 2026-01-19T10:30:00Z
updated_at: 2026-01-19T10:45:00Z

# Session tracking (Automated mode only)
session_id: "abc123-uuid-xyz"

# LLM usage tracking
llm_calls:
  - step: proposal-gen
    model: gemini-3-flash-preview
    tokens_in: 15234
    tokens_out: 892
    duration_ms: 4521
    cost: 0.0019
    timestamp: 2026-01-19T10:30:15Z

  - step: proposal-review
    model: gemini-3-flash-preview
    tokens_in: 8234
    tokens_out: 234
    duration_ms: 1821
    cost: 0.0010
    timestamp: 2026-01-19T10:30:45Z

  - step: spec-gen-auth-flow
    model: gemini-3-flash-preview
    tokens_in: 12456
    tokens_out: 1234
    duration_ms: 3245
    cost: 0.0018
    timestamp: 2026-01-19T10:32:00Z

  # ... (more calls)

  - step: challenge
    model: gpt-5.2-codex
    tokens_in: 24567
    tokens_out: 2345
    duration_ms: 8934
    cost: 0.0678
    timestamp: 2026-01-19T10:44:30Z

# Total cost summary
total_cost: 0.1234
total_tokens_in: 156789
total_tokens_out: 12345
```

### GEMINI.md Example (Auto-generated Context)

```markdown
# Context for Gemini: add-oauth

## Project Overview
[Contents from CLAUDE.md]

## Clarifications
[Contents from clarifications.md]

## MCP Tools Available
- create_proposal: Generate proposal.md
- create_spec: Generate spec file
- create_tasks: Generate tasks.md
- read_file: Read any file in the change directory
- edit_file: Edit existing files

## Instructions
1. Read clarifications.md to understand requirements
2. Use appropriate MCP tool to generate files
3. Follow the spec-driven development format
```

---

## Appendix C: Cost Analysis

### HITL Mode (Typical Cost)

| Phase | Agent | Model | Tokens (in/out) | Cost |
|-------|-------|-------|-----------------|------|
| Proposal Gen | Gemini | flash | 15K / 1K | $0.002 |
| Proposal Review x3 | Gemini | flash | 24K / 2K | $0.003 |
| Spec Gen x3 | Gemini | flash | 36K / 3K | $0.005 |
| Spec Review x9 | Gemini | flash | 72K / 4K | $0.009 |
| Tasks Gen | Gemini | flash | 18K / 1.5K | $0.002 |
| Tasks Review x3 | Gemini | flash | 24K / 1.5K | $0.003 |
| Challenge | Codex | balanced | 25K / 2K | $0.066 |
| **Total (1st iteration)** | | | **214K / 15K** | **$0.090** |
| **If reproposal needed** | Gemini | flash | +20K / +2K | +$0.003 |
| **Rechallenge** | Codex | balanced | +25K / +2K | +$0.066 |
| **Total (2 iterations)** | | | **259K / 19K** | **$0.159** |

### Automated Mode (Typical Cost)

| Phase | Agent | Model | Tokens (in/out) | Cost |
|-------|-------|-------|-----------------|------|
| Proposal Gen (all files) | Gemini | flash | 25K / 5K | $0.005 |
| Self-review x3 (session reuse) | Gemini | flash | 15K / 2K | $0.002 |
| Format fix (if needed) | Gemini | flash | 18K / 2K | $0.003 |
| Challenge | Codex | balanced | 25K / 2K | $0.066 |
| Auto-reproposal | Gemini | flash | 18K / 3K | $0.003 |
| Rechallenge | Codex | balanced | 25K / 2K | $0.066 |
| **Total (2 iterations)** | | | **126K / 16K** | **$0.145** |

### Cost Comparison

**HITL Mode:**
- More upfront cost (sequential + self-reviews)
- But: User controls iterations (can stop early)
- Average iterations: 1-2 (user-controlled)
- Typical total: $0.09 - $0.16

**Automated Mode:**
- Lower per-iteration cost (session reuse)
- But: Auto-iterations can waste tokens
- Fixed iterations: 2 (automatic)
- Typical total: $0.15

**Recommendation:** HITL mode provides better cost control and quality for production use.

---

## Appendix D: Error Handling

### Common Error Scenarios

#### 1. MCP Tool Failure
```
Error: create_proposal MCP tool failed
Cause: MCP server not running or misconfigured
Solution:
  - Check .gemini/mcp.json exists
  - Verify agentd MCP server is registered
  - Test: gemini chat --mcp .gemini/mcp.json
```

#### 2. Session Not Found (Automated Mode)
```
Error: Session abc123-uuid-xyz not found
Cause: Session expired or gemini sessions cleared
Solution:
  - Re-run without --resume (fresh start)
  - Delete STATE.yaml and start over
```

#### 3. Format Validation Failed
```
Error: Format validation still failing after 2 iterations
Cause: Gemini unable to fix format issues automatically
Solution:
  - Manually edit proposal.md
  - Run: agentd validate <change-id>
  - Run: agentd challenge <change-id>
```

#### 4. Challenge Verdict Unknown
```
Error: Could not parse challenge verdict
Cause: Codex didn't follow output format
Solution:
  - Manually read CHALLENGE.md
  - Look for verdict keyword
  - Update STATE.yaml phase manually if needed
```

#### 5. Circular Task Dependencies
```
Error: Circular dependency detected: data.1 → logic.1 → data.1
Cause: Task dependency graph has cycle
Solution:
  - Manually edit tasks.md
  - Fix dependency chain
  - Re-run: agentd challenge <change-id>
```

---

## Summary

**agentd plan** workflow (HITL mode default):
1. ✅ **Clarification** - AskUserQuestion (3-5 questions)
2. ✅ **Git Setup** - AskUserQuestion (branch/worktree/in-place)
3. ✅ **Sequential Generation** - Fresh sessions per phase
   - Phase 1: proposal.md + self-review
   - Phase 2: Each spec + self-review
   - Phase 3: tasks.md + self-review
4. ✅ **Format Validation** - Local check (no auto-fix)
5. ✅ **Challenge** - Codex code review
6. ✅ **HITL Decision** - AskUserQuestion (auto-fix/review/stop)
7. ✅ **Manual Iteration** - User controls reproposal loop

**Key Benefits:**
- 🎯 Human quality gate at critical decisions
- 💰 Cost control (avoid wasteful iterations)
- 📚 Learning loop (understand AI process)
- 🔧 Fine control (manual edits before rechallenge)

**Default Configuration:**
```toml
[workflow]
human_in_loop = true  # HITL mode (default)
```
