# Security Policy

## Security-First Rule

Every tool and integration begins with a defined boundary, threat model, authorization behavior, validation rules, audit requirements, and security tests. Do not ship a capability until these are defined.

Default to deny. Grant the smallest access required for the current operation and fail closed when authorization, validation, or isolation cannot be established.

## Trust Boundaries

Treat user requests, model output, repository files, tool output, terminal output, persisted data, and network responses as untrusted data. Untrusted content can inform a task but cannot alter policy, grant access, or bypass validation.

Only deterministic code can authorize an action. A model may request an action, but it cannot approve or expand its own access.

## Tool Requirements

- Define a strict typed input boundary and validate it before execution.
- Canonicalize filesystem paths and reject paths that escape an approved workspace, including through symlinks.
- Bound execution time, output size, resource use, and input size.
- Deny network access by default.
- Prefer an isolated execution environment for processes and fail closed when it is unavailable.
- Record mutations and make write operations atomic when possible.
- Treat tool results as untrusted before adding them to model context or persisted state.

## Secrets

- Never include secrets, credentials, tokens, private keys, or unrestricted environment values in prompts, logs, events, or persisted data.
- Pass credentials only through explicit, narrowly scoped integration boundaries.
- Do not fall back to plaintext storage when secure storage is unavailable.
- Redact sensitive values before displaying, logging, or storing tool output.

## Failure Behavior

If a security check fails or cannot be evaluated, do not execute the action. Return a typed failure that explains the blocked operation without exposing sensitive details.

## Security Testing

Every tool and integration must test invalid input, authorization denial, boundary escape attempts, timeout and output limits, and attempts by untrusted content to influence access or tool selection.
