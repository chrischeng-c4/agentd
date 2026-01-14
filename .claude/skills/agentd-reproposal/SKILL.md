---
name: agentd:reproposal
description: Refine proposal based on challenge feedback
user-invocable: true
---

# /agentd:reproposal

Automatically fix issues found in challenge.

## Usage

```bash
agentd reproposal <change-id>
```

## Example

```bash
agentd reproposal add-oauth
```

## What it does

1. Reads original proposal and `CHALLENGE.md`
2. Calls Gemini to fix all issues
3. Regenerates proposal files
4. Shows what was fixed

## Next step

Run `/agentd:challenge <change-id>` again to verify fixes.

Repeat until no HIGH severity issues remain.
