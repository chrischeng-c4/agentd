---
change: fix-workflow-bugs
date: 2026-01-23
---

# Clarifications

## Q1: Tasks Fix
- **Question**: tasks.md frontmatter 問題要如何修復？
- **Answer**: 修復 create_tasks MCP tool
- **Rationale**: 在 Rust 的 create_tasks 函數中確保輸出包含 YAML frontmatter，這是最可靠的方式

## Q2: Review Fix
- **Question**: REVIEW.md 模板問題要如何處理？
- **Answer**: Codex 應該用 MCP 寫入 REVIEW.md
- **Rationale**: 新增 create_review MCP tool 讓 Codex 直接寫入結構化的 review 結果

## Q3: Pipe Fix
- **Question**: stdin pipe 錯誤要如何處理？
- **Answer**: 加強錯誤處理
- **Rationale**: 在 subprocess 通訊時加入 retry 和 graceful error handling，避免誤導性 exit code

## Q4: Git Workflow
- **Question**: Git workflow 偏好？
- **Answer**: In place
- **Rationale**: 在目前 branch 上直接修改，簡化流程

