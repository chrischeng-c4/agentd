---
title: HTTP MCP Server
source: Internal implementation
created: 2026-01-21
updated: 2026-01-21
---

# HTTP MCP Server

Global HTTP MCP server with multi-project support for Agentd.

## Overview

The HTTP MCP server solves the stdout buffering issue that occurs with stdio transport in Pipe environments. It runs as a single global daemon on port 3000, with project isolation via HTTP headers.

## Architecture

```
┌─────────────────┐
│  Claude Code    │──┐
└─────────────────┘  │  HTTP + X-Agentd-Project header
                     │
┌─────────────────┐  │    ┌──────────────────────┐
│  Other Tools    │──┼───>│  HTTP MCP Server     │
└─────────────────┘  │    │  (localhost:3000)    │
                     │    │                      │
                     │    │  ┌───────────────┐  │
                     └───>│  │  Registry     │  │
                          │  │  (projects)   │  │
                          │  └───────────────┘  │
                          └──────────────────────┘
                                    │
                        ┌───────────┴───────────┐
                        │                       │
                   ┌────▼────┐           ┌────▼────┐
                   │ Project │           │ Project │
                   │    A    │           │    B    │
                   └─────────┘           └─────────┘
```

## Key Features

1. **Single Server**: One global server instance on port 3000
2. **Multi-Project**: Multiple projects registered with the same server
3. **Project Isolation**: Each project operates in its own directory context
4. **Dynamic Registry**: Auto-reloads project registrations on each request
5. **Auto-Configuration**: Automatically updates client configs

## Transport Protocol

Uses **Streamable HTTP** transport (MCP 2024-11-05 spec):
- HTTP POST to `/mcp` endpoint
- JSON-RPC 2.0 over HTTP
- Project identified by `X-Agentd-Project` header
- No buffering issues (unlike stdio)

## CLI Commands

### Start Server and Register Project

```bash
cd your-project
agentd mcp-server start

# With auto-configuration
agentd mcp-server start --update-clients

# In daemon mode (background)
agentd mcp-server start --daemon

# Custom port
agentd mcp-server start --port 8080
```

### List Registered Projects

```bash
agentd mcp-server list
```

Output:
```
MCP Server Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Port:    3000
  PID:     12345
  Started: 2026-01-21 10:00:00
  Status:  Running

Registered Projects
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ● my-project
    Path: /Users/user/projects/my-project
    Registered: 2026-01-21 10:00:15
```

### Stop/Unregister Project

```bash
# Unregister current directory project
agentd mcp-server stop

# Unregister specific project
agentd mcp-server stop my-project
```

### Shutdown Server

```bash
agentd mcp-server shutdown
```

## Client Configuration

### Claude Code

Configure MCP server in `.claude/settings.json` or Claude Code's config:

```json
{
  "mcpServers": {
    "agentd": {
      "url": "http://localhost:3000/mcp",
      "transport": "http",
      "headers": {
        "X-Agentd-Project": "my-project",
        "X-Agentd-Cwd": "/Users/user/projects/my-project"
      },
      "timeout": 30000
    }
  }
}
```

Use `--update-clients` flag to auto-generate these configurations.

## Testing

### Health Check

```bash
curl http://localhost:3000/health
# Returns: OK
```

### Initialize Request

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-Agentd-Project: my-project" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {}
  }'
```

### List Tools

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-Agentd-Project: my-project" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
  }'
```

### Call Tool

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -H "X-Agentd-Project: my-project" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "list_knowledge",
      "arguments": {}
    }
  }'
```

## Registry File

Located at `~/.agentd/registry.json`:

```json
{
  "server": {
    "pid": 12345,
    "port": 3000,
    "started_at": "2026-01-21T10:00:00Z"
  },
  "projects": {
    "my-project": {
      "path": "/Users/user/projects/my-project",
      "registered_at": "2026-01-21T10:00:15Z"
    },
    "another-project": {
      "path": "/Users/user/projects/another-project",
      "registered_at": "2026-01-21T10:05:30Z"
    }
  }
}
```

## Implementation Details

### Project Context Switching

When a request arrives:
1. Extract `X-Agentd-Project` header
2. Reload registry from disk (picks up new registrations)
3. Look up project path in registry
4. Change working directory to project path
5. Execute MCP request in project context
6. Restore original directory

### Dynamic Registry Reloading

The server reloads the registry from disk on every request, allowing:
- New projects to be registered without server restart
- Projects to be unregistered dynamically
- Multiple `agentd mcp-server start` calls from different projects

### Process Management

The server runs as a background daemon process:
- Parent process: `agentd mcp-server start` (registers project, exits)
- Daemon process: `agentd mcp-server run --port 3000` (actual HTTP server)
- Registry tracks daemon PID for lifecycle management

## Comparison with Stdio Transport

| Feature | Stdio | HTTP |
|---------|-------|------|
| Buffering issues | Yes (in Pipe) | No |
| Multi-project | No | Yes |
| Global instance | No | Yes |
| File I/O for LLMs | Yes (/tmp) | No |
| Port conflicts | N/A | Possible (fixed port) |

## Troubleshooting

### Server won't start

```bash
# Check if already running
agentd mcp-server list

# If dead, clean up
rm ~/.agentd/registry.json
agentd mcp-server start
```

### Project not found

```bash
# Re-register the project
cd your-project
agentd mcp-server start
```

### Port already in use

```bash
# Use different port
agentd mcp-server start --port 8080
```

## Related

- [MCP Index](index.md)
- [Claude Code MCP](claude-mcp.md)
- [Dynamic Configuration](dynamic-config.md)
