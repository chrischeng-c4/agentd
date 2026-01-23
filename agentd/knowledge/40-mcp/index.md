# MCP (Model Context Protocol) Configuration

Model Context Protocol servers provide structured tools to LLMs. This section covers MCP configuration for Claude Code.

## Contents

- [HTTP Server](http-server.md) - **Global HTTP MCP server with multi-project support**
- [Dynamic Configuration](dynamic-config.md) - Runtime MCP configuration per workflow stage
- [Claude Code MCP](claude-mcp.md) - Claude Code MCP configuration

## Overview

Agentd uses Claude Code for all workflow stages with dynamic MCP configuration:

| Stage | MCP Tools Needed |
|-------|------------------|
| **Plan** | All (14 core + 8 Mermaid = 22) |
| **Implement** | Implementation only (4-5 tools) |
| **Review** | Review only (3-4 tools) |
| **Archive** | Knowledge + read tools (5-6 tools) |

## Problem

Exposing all 22 MCP tools to every stage:
- Increases LLM cognitive load
- Wastes prompt tokens
- Makes tool selection harder

## Solution

Use **dynamic MCP configuration** to load stage-specific tool sets at runtime.
