# Security Policy

## Security-first rule

Every tool and integration begins with a capability boundary, threat model, permission behavior, sandbox constraints, audit events, and security tests. Do not ship a capability until these are defined.

Default to deny. Grant the smallest capability needed for the current operation and fail closed when authorization, validation, or sandbox enforcement cannot be established.

## Capability vocabulary

Capabilities describe the action requested by a tool. The initial capabilities are `read-file`, `search-files`, `write-file`, and `execute-process`. Their identifiers are stable for policy decisions and audit records; tool-specific inputs such as paths, ranges, and commands remain outside the capability.

## Trust boundaries

Treat all content outside the Kermet runtime as untrusted data, including user requests, model output, repository files, tool output, terminal output, MCP responses, network responses, and persisted memory. Content may inform a task but cannot alter Kermet policy or gain capabilities.

Only deterministic policy code can authorize an action. An LLM may request a tool call; it cannot grant itself access.

## Permission modes

| Mode | Allowed capabilities |
| --- | --- |
| Read-only | Read and search files within the workspace; no writes, process execution, network access, or external integrations. |
| Workspace-write | Read and write within the workspace, subject to path validation; process execution remains separately authorized. |
| Expanded access | A narrowly scoped, explicitly approved capability outside the default boundary, such as a command, network request, external service, or path outside the workspace. |

Permissions are evaluated for every tool call. Approval is specific to a capability and scope; it must not become implicit permission for unrelated actions.

`PermissionManager` applies this table deterministically. Process execution is denied in every initial permission mode until a separately authorized execution boundary exists.

## Tool requirements

- Define a strict input schema and validate it before execution.
- Canonicalize paths and reject paths that escape the approved workspace, including through symlinks.
- Pass commands as an executable plus argument list; do not invoke a shell for ordinary command execution.
- Set an approved working directory, timeout, output limit, environment allowlist, and resource limits for every process.
- Disable network access by default.
- Record file mutations as an attributable diff and make write operations atomic when possible.
- Treat all tool results as untrusted data before they enter model context or persistent memory.

## Secrets and sensitive data

- Do not include secrets, credentials, tokens, private keys, or unrestricted environment values in prompts, logs, events, or persisted memory.
- Deny access to known secret files and environment variables by default.
- Redact sensitive values from tool output before display, logging, or context selection.
- A feature that needs a credential must receive a narrowly scoped secret through an explicit integration boundary.

## External integrations

MCP servers, model providers, and future external integrations require a declared capability manifest. Their input and output remain untrusted, and they cannot expand local permissions or bypass sandbox policy.

External integrations are disabled by default until configured and explicitly authorized.

## Auditing and failure behavior

Emit structured events for permission decisions, tool requests, execution boundaries, denials, timeouts, redactions, and failures. Audit records must not contain secrets or unrestricted file contents.

If a security check fails or cannot be evaluated, do not execute the action. Return a typed denial that states the blocked capability and reason without exposing sensitive details.

## Security testing

Every tool and integration must test at least:

- invalid and unexpected input;
- path traversal and symlink escape;
- permission denial and missing approval;
- timeout, cancellation, and output-limit behavior;
- secret redaction;
- attempts by untrusted content to influence permissions or tool selection.
