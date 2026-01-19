# Knowledge Base

This directory contains descriptive documentation about how this system works.

- **Specs** define what should be built (prescriptive)
- **Knowledge** documents how it currently works (descriptive)

## Structure

Use numbered prefixes for ordering:
- `00-09`: System-level (architecture, principles)
- `10+`: Domain modules

Example:
```
knowledge/
  00-architecture/
    index.md
    01-overview.md
    02-design-principles.md
  10-auth/
    index.md
    01-oauth-flow.md
```

## Contents

<!-- Add entries as you document the system -->

## Usage

LLM tools can read knowledge via MCP:
- `list_knowledge` - List all knowledge files
- `read_knowledge` - Read specific file
