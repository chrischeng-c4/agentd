# MCP (Model Context Protocol) Configuration

Model Context Protocol servers provide structured tools to LLMs. This section covers MCP configuration strategies for different LLM providers.

## Contents

- [Dynamic Configuration](dynamic-config.md) - Runtime MCP configuration per workflow stage
- [Claude Code MCP](claude-mcp.md) - Claude Code MCP configuration
- [Codex MCP](codex-mcp.md) - OpenAI Codex MCP configuration
- [Gemini MCP](gemini-mcp.md) - Google Gemini MCP configuration

## Overview

Agentd uses different LLM providers for different workflow stages:

| Stage | Provider | Context Window | MCP Tools Needed |
|-------|----------|----------------|------------------|
| **Plan** | Gemini | 2M tokens | All (14 core + 8 Mermaid = 22) |
| **Implement** | Claude | 200K tokens | Implementation only (4-5 tools) |
| **Review** | Codex | 400K tokens | Review only (3-4 tools) |
| **Archive** | Gemini | 2M tokens | Knowledge + read tools (5-6 tools) |

## Problem

Exposing all 22 MCP tools to every stage:
- Increases LLM cognitive load
- Wastes prompt tokens
- Makes tool selection harder

## Solution

Use **dynamic MCP configuration** to load stage-specific tool sets at runtime.
