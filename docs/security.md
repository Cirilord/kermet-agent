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

### Codex OAuth

Codex authentication may contact only `https://auth.openai.com` to exchange or refresh a token. The authorization callback is accepted only from the loopback listener, and the authorization code is accepted only when its `state` matches the request that created it. The flow uses PKCE with `S256`, limits token responses to 16 KiB, and applies a finite request timeout. Access and refresh tokens must never be logged, included in errors, or added to model context.

Configured Codex inference requests use `https://chatgpt.com/backend-api/codex/responses` over HTTPS, with a finite timeout and a 2 MiB response limit. The provider sends only the selected model configuration, task, declared tool schema, and bounded tool output required for the current turn. It must validate every streamed response and function-call argument before creating an internal tool request.

Persist provider credentials only through the operating system credential store. Kermet uses a stable service name and a provider/profile account name, and it does not fall back to a plaintext file when the credential store is unavailable. The only alternative is an explicit ephemeral store that retains credentials in process memory. Deleting credentials must be idempotent.

User configuration files may contain provider names, model identifiers, and reasoning settings, but never credentials or token material.

Thread history is local, plaintext application data containing the workspace path and user and assistant messages. It must never contain credentials or secret material. Store it outside the workspace under the user-owned Kermet directory, validate its schema and identifier before use, and keep credential persistence separate in the operating system credential store.

### Hub sessions

The Hub binds only to the loopback interface. Each server process generates an in-memory, single-use bootstrap token and prints the bootstrap URL to the terminal. The bootstrap route exchanges that token for an in-memory `HttpOnly`, `SameSite=Strict` session cookie and redirects to the clean home URL. All Hub routes other than the bootstrap route deny requests without the matching session cookie. Sessions disappear when the Hub process exits.

The local Hub currently uses HTTP on loopback, so its cookie cannot include `Secure`. The session-cookie boundary takes an explicit transport value and adds `Secure` whenever the Hub is served over HTTPS. Do not expose the Hub beyond loopback until HTTPS is configured.

### Future Hub mutations

The Hub has no mutation routes. Access control distinguishes read routes from mutation routes: authenticated reads are allowed, while mutations fail closed until mutation protection exists. A new route that creates, changes, or deletes data must use the mutation access boundary and implement all of the following in the same change:

- require the valid session;
- validate an expected `Origin` before parsing or changing data;
- validate a server-generated CSRF token using the synchronizer-token pattern;
- emit an audit event with the action, target, session identifier or a non-secret session correlation value, result, and failure reason when applicable;
- use a non-`GET` method and retain the request limits appropriate to its input.

CSRF tokens, origin validation, and mutation audit events are intentionally not implemented while every Hub route is read-only. They must not be added as optional middleware or bypassed by an individual route.

Every Hub response disables browser caching and includes content-security, anti-framing, anti-sniffing, referrer, opener, and permissions policies. The Hub embeds its behavior script from the compile-time `src/hub/assets/hub.js` asset. Its content-security policy authorizes that script only through its SHA-256 hash; it does not permit inline scripts generally.

The Hub currently exposes read-only `GET` routes, so it rejects request bodies. It also rejects request targets larger than 2 KiB, more than 32 request headers, and request headers larger than 8 KiB in total. These limits must be reconsidered deliberately before introducing a route that accepts request data.

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
