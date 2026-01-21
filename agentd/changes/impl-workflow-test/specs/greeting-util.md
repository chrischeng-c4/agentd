---
id: greeting-util
type: spec
version: 1
created_at: 2026-01-22T00:30:00Z
updated_at: 2026-01-22T00:30:00Z
status: approved
---

# Spec: Greeting Utility

## Overview
A simple greeting utility function for testing purposes.

## Scenarios

### Scenario: WHEN greet is called with a name THEN returns formatted greeting
**GIVEN** the greeting module is available
**WHEN** `greet("World")` is called
**THEN** it returns `"Hello, World!"`

### Scenario: WHEN greet is called with empty string THEN returns greeting with empty name
**GIVEN** the greeting module is available
**WHEN** `greet("")` is called
**THEN** it returns `"Hello, !"`

## Implementation Notes
- Function signature: `pub fn greet(name: &str) -> String`
- Use format! macro for string formatting
- Include unit tests in the same file
