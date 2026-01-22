---
change: agent-agnostic-prompts
date: 2026-01-22
---

# Clarifications

## Q1: Agent 範圍
- **Question**: XML structured input 是否要支援所有 agents？
- **Answer**: All agents (Gemini, Codex, Claude Code)
- **Rationale**: 統一格式可以減少維護成本，讓所有 agents 使用相同的 prompt 結構

## Q2: System Prompt
- **Question**: System prompt 注入方式？
- **Answer**: Prepend to user prompt
- **Rationale**: 統一 prepend 到 user prompt，不依賴各 CLI 的 native 支援，保持一致性

## Q3: Session Reuse
- **Question**: 要在這次重構中處理 session reuse 嗎？
- **Answer**: 拿掉 session reuse 需求
- **Rationale**: 三個 agent 本身就會多輪對話(session reuse)，phase 之間透過 artifact 傳遞 context

## Q4: Git Workflow
- **Question**: Git workflow 偏好？
- **Answer**: New branch
- **Rationale**: 使用 git checkout -b agentd/agent-agnostic-prompts 進行開發

