# Development Workflow

## Prerequisites

- Install the current stable Rust toolchain.
- Install the Clippy and rustfmt components.
- Install Lefthook and run `lefthook install`.
- Install Cocogitto.
- Run commands from the repository root.

## Local commands

```sh
cargo run
cargo test
cargo fmt --check
cargo clippy -- -D warnings
cog verify --file .git/COMMIT_EDITMSG
```

Run the complete validation suite before considering a change complete:

```sh
cargo fmt --check &&
cargo clippy -- -D warnings &&
cargo test
```

Cocogitto validates Conventional Commit messages. Lefthook runs formatting, Clippy, and tests before commits, then validates commit history before pushes. Do not bypass a hook; fix the reported issue or update the relevant check deliberately.

Commit messages follow Conventional Commits.

```text
feat(tools): add a tool registry
```

## Development flow

1. Read `AGENTS.md` and the documentation relevant to the change.
2. Define the intended behavior and failure cases before implementation.
3. Implement the smallest complete change.
4. Add or update deterministic tests.
5. Run formatting, lints, and tests.
6. Update documentation when the change introduces a lasting decision, interface, workflow, or security boundary.

## Definition of done

A change is complete when:

- its behavior and failure cases are covered by tests;
- `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` pass;
- it preserves the fail-fast rules in `docs/conventions.md`;
- tool and integration changes satisfy `docs/security.md`;
- relevant documentation is current.

## Dependencies

Add a dependency only when the standard library or existing dependencies cannot meet the requirement clearly.

Before adding one:

1. Confirm its maintenance status, license, and security posture.
2. Prefer small, focused crates.
3. Document why it is needed when it establishes a lasting architectural dependency.

`ureq` is the synchronous HTTPS client for provider adapters. Codex OAuth uses it with platform certificate verification, HTTPS-only requests, bounded responses, and a timeout.

`keyring` stores persistent provider credentials in the operating system credential store. Tests must use the ephemeral credential store rather than a real user keyring.

`toml` parses the versioned, non-secret user configuration used to resolve model profiles.

`axum` provides HTTP routing for the local hub server, while `maud` renders its server-side HTML. `tokio` owns the hub server runtime and listener.

The Hub loads Tailwind CSS 4.3.3, jQuery 4.0.0, and Lucide 1.61.0 from their CDNs using exact version URLs. It embeds its behavior script at compile time from `src/hub/assets/hub.js` and keeps the Base Nova semantic theme tokens in the server-rendered base layout, so it does not require a Node.js toolchain or a `package.json`.

## Persisted state

- Version every persisted schema.
- Make migrations explicit and testable.
- Preserve backwards compatibility or provide a documented migration path.
- Do not modify persisted data silently or destructively.

## Commits and pull requests

- Keep changes focused and independently reviewable.
- Explain the behavior change, validation performed, and relevant risks.
- Do not mix refactors with behavior changes unless they must be delivered together.
