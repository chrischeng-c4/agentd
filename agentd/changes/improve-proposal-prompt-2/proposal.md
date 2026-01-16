# Change: improve-proposal-prompt-2

## Summary
Enhance the `agentd:proposal` Gemini command prompt with advanced prompt engineering techniques to improve proposal quality, consistency, and technical depth.

## Why
The current proposal generation prompt is relatively simple and can sometimes produce vague or surface-level technical designs. By incorporating techniques such as persona adoption, Chain-of-Thought reasoning, structured instruction sets, and explicit few-shot examples, we can guide the AI to perform deeper codebase analysis and produce more rigorous specifications and task lists.

## What Changes
- **Persona Adoption**: Define the agent as a "Senior Staff Systems Engineer" with expertise in Rust and distributed systems.
- **Chain-of-Thought (CoT)**: Require the agent to explicitly state its plan and reasoning before generating files.
- **Structured Instructions**: Organize the prompt using clear sections and hierarchical instructions.
- **Enhanced Constraints**: Explicitly forbid "TBD", "placeholder", and generic descriptions.
- **Refined Abstractions**: Provide better guidance on how to use Mermaid and JSON Schema for complex scenarios.
- **Updated `proposal.toml`**: Replace the current `prompt` value with the engineered version.

## Impact
- Affected specs: `proposal-prompt.md`
- Affected code: `.gemini/commands/agentd/proposal.toml`
- Breaking changes: No