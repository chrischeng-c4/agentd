---
change: merge-viewer-to-mcp
date: 2026-01-22
---

# Clarifications

## Q1: Server 架構
- **Question**: 合併後的 HTTP server 架構要怎麼設計？
- **Answer**: 單一 port 多路由
- **Rationale**: MCP API (/mcp/*) 和 Viewer UI (/*) 共用同一個 port，簡化部署和使用

## Q2: View 命令
- **Question**: 現有的 standalone `agentd view` 命令要如何處理？
- **Answer**: 移除，改用瀏覽器開 MCP server
- **Rationale**: 簡化架構，用戶直接訪問 http://localhost:<port>/view/<change-id>

## Q3: 命令重命名
- **Question**: MCP server 的命令要如何命名？
- **Answer**: 改名為 agentd server start
- **Rationale**: 反映新功能 - 包含 plan viewer 和 MCP server 兩種功能

