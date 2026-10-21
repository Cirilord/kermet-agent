# Engineering Conventions

## General

- Use English for source code, comments, documentation, identifiers, and user-facing text.
- Keep modules small and give each one a clear responsibility.
- Make dependencies explicit through function arguments and constructors.
- Avoid global mutable state.
- Introduce an abstraction only when it has at least two concrete use cases.

## Rust

- Use stable Rust.
- Keep `cargo fmt --check` and `cargo clippy -- -D warnings` clean.
- Declare an explicit return type on every function, including `-> ()`.
- End every function with an explicit `return`; do not use implicit tail-expression returns.
- Prefer domain types and enums over primitive strings and booleans.
- Use `Result` for expected failures and typed error enums at module boundaries.
- Do not use `unwrap()` or `expect()` in production paths.
- Use `thiserror` for library errors and `anyhow` only at application boundaries.
- Avoid `unsafe`; document and isolate every required use.

## Errors and fail-fast behavior

- Validate input at the boundary where it enters the system.
- Fail immediately on invalid state, broken invariants, unsupported operations, and missing required configuration.
- Do not silently coerce invalid input or substitute fallback values.
- Include actionable context in errors without exposing secrets.
- Make retries bounded, explicit, and observable.

## Async and concurrency

- Give asynchronous work an owner, cancellation behavior, and a timeout when applicable.
- Do not detach background work without tracking its result.
- Avoid shared mutable state; use message passing or synchronization primitives deliberately.
- Preserve ordering when tool calls or file mutations depend on each other.

## Public interfaces

- Keep provider and tool interfaces provider-agnostic.
- Validate external payloads before converting them to domain types.
- Version persisted schemas and migrations.
- Document invariants and side effects on public APIs.

## Tests

- Test deterministic behavior without LLM calls whenever possible.
- Use fakes for models, tools, clocks, and filesystem boundaries.
- Add a regression test for every fixed bug.
- Test errors and permission denials as carefully as successful paths.

## Observability

- Emit structured events for state transitions, model requests, tool calls, retries, and failures.
- Never log secrets, credentials, or unrestricted file contents.
