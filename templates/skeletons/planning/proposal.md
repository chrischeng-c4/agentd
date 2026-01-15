# Change: {{change_id}}

<meta>
  <purpose>PRD - Product Requirements Document</purpose>
  <constraint>Describes WHAT and WHY, not HOW</constraint>
</meta>

<section id="summary" required="true">
## Summary

[Brief 1-2 sentence description of the change]

<quality>
- Be specific, not vague ("Add OAuth2 with Google" not "Add authentication")
- State the main outcome, not the process
- Should fit in a commit message title
</quality>
</section>

<section id="why" required="true">
## Why

[Problem statement and motivation]

<quality>
- Explain the business or user impact
- Quantify if possible ("blocks 3 enterprise customers")
- Link to issues or user feedback if available
</quality>
</section>

<section id="what-changes" required="true">
## What Changes

[Bullet points of what will be added/modified/removed]

<quality>
- List concrete additions/modifications
- Group by area (e.g., API, UI, Database)
- Be specific about file paths or components
</quality>
</section>

<section id="impact" required="true">
## Impact

- Affected specs: [list spec names that will be created/modified]
- Affected code: [list file paths or systems]
- Breaking changes: [Yes/No - if Yes, explain migration path]

<quality>
- List specific file paths, not vague areas
- Identify dependencies that might be affected
- Note any backwards compatibility concerns
</quality>
</section>
