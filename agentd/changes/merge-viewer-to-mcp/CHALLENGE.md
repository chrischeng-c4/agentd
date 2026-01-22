# Challenge: merge-viewer-to-mcp

## Feedback

### [HIGH] Missing Dashboard Specification
The proposal mentions that `GET /` should be a "Dashboard listing registered projects and active changes," but this is not reflected in the Requirements, Flow, or Acceptance Criteria of the specification.
- **Action**: Add a requirement (R6) for the dashboard and include a corresponding scenario in the acceptance criteria.

### [HIGH] UX Regression: Removal of `view` command
Removing the `agentd view <change_id>` command without a CLI-based replacement forces users to manually construct and open URLs in their browser.
- **Action**: Consider adding `agentd server open <project> <change>` or `agentd server view <project> <change>` as a convenience command that starts the server (if not running) and opens the default browser.

### [MEDIUM] API Path Robustness in Frontend
The spec mentions updating `app.js` to use relative paths. However, with nested routes like `/view/:project/:change/`, relative paths in `app.js` (e.g., `fetch('api/...')`) will resolve differently depending on whether there's a trailing slash or not.
- **Action**: Update the spec to describe a mechanism for the server to inject the base API path into the HTML template, or use absolute paths from the root (e.g., `/view/:project/:change/api/...`).

### [MEDIUM] Static Asset Path Inconsistency
The spec proposes a `/static` path for assets. Existing viewer code likely expects assets at the root or relative to the current page. 
- **Action**: Clarify in the tasks how the HTML template for the viewer will be updated to point to the new `/static` location.

### [LOW] Registry Error Handling
The flow assumes the Registry always resolves project names successfully.
- **Action**: Add an error case to the sequence diagram and acceptance criteria for when a project or change ID is not found in the registry.

## Verdict
**REJECTED**

The proposal is conceptually sound, but the specification lacks detail on the dashboard and contains a significant UX regression that should be addressed before implementation.
