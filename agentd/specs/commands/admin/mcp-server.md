# Specification: MCP-Server Command

## Overview

The `mcp-server` command starts an MCP (Model Context Protocol) server that exposes Agentd functionality as tools for Claude Desktop and other MCP clients. It enables integration with Claude Desktop's tool ecosystem.

## Requirements

### R1: Server Startup
- Start HTTP/stdio MCP server
- Register Agentd tools (create_proposal, create_spec, create_tasks, etc.)
- Listen for MCP protocol requests
- Run until interrupted (Ctrl+C)

### R2: Tool Registration
- Expose proposal creation tools
- Expose validation tools
- Expose knowledge base tools
- Provide JSON Schema definitions for each tool

### R3: Request Handling
- Parse MCP tool invocation requests
- Execute corresponding Agentd operations
- Return results in MCP response format
- Handle errors gracefully

### R4: Integration
- Compatible with Claude Desktop MCP configuration
- Follows MCP protocol specification
- Supports stdio and HTTP transports

## Command Signature

```bash
agentd mcp-server
```

**Arguments:**
- None

**Options:**
- None

## Exit Codes

- `0`: Success (server shutdown gracefully)
- `1`: Error (server startup failed, port in use)

## Examples

```bash
$ agentd mcp-server
MCP Server starting...
Registered tools:
  - create_proposal
  - create_spec
  - create_tasks
  - validate_change
  - read_knowledge
  - list_knowledge
Server listening on stdio...
^C
Server shutdown.
```

## Claude Desktop Configuration

```json
{
  "mcpServers": {
    "agentd": {
      "command": "agentd",
      "args": ["mcp-server"]
    }
  }
}
```

## Notes

- Designed for use with Claude Desktop
- Runs in foreground - use process manager for background
- No authentication - assumes local trusted environment
- Tools match MCP naming conventions
- Server uses stdio by default for Claude Desktop compatibility
