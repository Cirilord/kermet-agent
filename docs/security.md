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
| Expanded access | A narrowly scoped, explicitly approved capability outside the default boundary, such as process execution, a network request, external service, or a path outside the workspace. |

Permissions are evaluated for every tool call. Approval is specific to a capability and scope; it must not become implicit permission for unrelated actions.

`PermissionManager` applies this table deterministically. Process execution is denied in `read-only` and `workspace-write`; it requires explicit `expanded-access`.

### Temporary Bash execution boundary

The initial `bash` tool has no OS process sandbox. It is available only with `expanded-access`, starts with the workspace as its working directory, accepts no standard input, clears its environment except for `PATH`, times out after 30 seconds, and bounds each output stream to 32 KiB. Those controls do not restrict filesystem access, child processes, or network access. The local Hub currently creates turns with `expanded-access` for testing; it must return to a restricted mode or gain a per-turn permission control when the shared sandbox boundary is introduced.

## Tool requirements

- Define a strict input schema and validate it before execution.
- Canonicalize paths and reject paths that escape the approved workspace, including through symlinks.
- Pass commands as an executable plus argument list; do not invoke a shell for ordinary command execution.
- Set an approved working directory, timeout, output limit, environment allowlist, and resource limits for every process.
- Disable network access by default. The temporary Bash tool is an explicit exception under `expanded-access` until sandbox enforcement is implemented.
- Record file mutations as an attributable diff and make write operations atomic when possible.
- Treat all tool results as untrusted data before they enter model context or persistent memory.

### Write tool

`write` creates or replaces an entire regular file. It cannot append, edit a range, create parent directories, or follow a path or parent symlink outside the workspace. It writes to a new sibling temporary file, syncs that file, and then renames it over the target. Its input content is limited to 256 KiB.

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

### Hub mutations

Authenticated Hub reads are allowed. The workspace-picker endpoint is a protected `POST` route that starts the current Kermet executable with the fixed hidden `hub --open-picker` command. The child process opens a native folder dialog and writes one bounded JSON result to standard output; it receives no browser-controlled arguments, no standard input, and no inherited standard error. The parent applies a two-minute timeout, kills an unfinished child, accepts at most 16 KiB of output, canonicalizes and validates the selected directory, and persists a workspace record. The route validates the session, exact loopback `Origin`, and a server-generated synchronizer CSRF token before starting the picker.

A new route that creates, changes, or deletes data must use the mutation access boundary and implement all of the following in the same change:

- require the valid session;
- validate an expected `Origin` before parsing or changing data;
- validate a server-generated CSRF token using the synchronizer-token pattern;
- emit an audit event with the action, target, session identifier or a non-secret session correlation value, result, and failure reason when applicable;
- use a non-`GET` method and retain the request limits appropriate to its input.

CSRF tokens, origin validation, and mutation audit events are mandatory for Hub mutation routes. They must not be optional middleware or bypassed by an individual route.

Every Hub response disables browser caching and includes content-security, anti-framing, anti-sniffing, referrer, opener, and permissions policies. The Hub embeds its behavior script from the compile-time `src/hub/assets/hub.js` asset. Its content-security policy authorizes that script only through its SHA-256 hash; it does not permit inline scripts generally.

The Hub accepts JSON request bodies only when creating a thread or adding a message. Each body is limited to 4 KiB; all other routes, including the folder-picker and initial-turn endpoints, reject non-empty bodies. It rejects request targets larger than 2 KiB, more than 32 request headers, and request headers larger than 8 KiB in total. These limits must be reconsidered deliberately before introducing a route that accepts request data.

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
