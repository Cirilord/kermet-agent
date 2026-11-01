# Development Workflow

## Prerequisites

- Install the current stable Rust toolchain with Clippy and rustfmt.
- Install Lefthook and run `lefthook install`.
- Install Cocogitto.
- Run commands from the repository root.

## Local Commands

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

Lefthook runs formatting, Clippy, and tests before commits. Cocogitto validates Conventional Commit messages. Do not bypass a hook; fix the reported issue or deliberately update the relevant check.

## Development Flow

1. Read `AGENTS.md` and the documentation relevant to the change.
2. Define intended behavior and failure cases before implementation.
3. Implement the smallest complete change.
4. Add or update deterministic tests.
5. Run formatting, lints, and tests.
6. Update documentation when a change introduces a lasting decision, interface, workflow, or security boundary.

## Definition of Done

A change is complete when its behavior and failure cases are covered by tests, the validation suite passes, and relevant documentation is current.

## Dependencies

Add a dependency only when the standard library or existing dependencies cannot meet the requirement clearly. Confirm its maintenance status, license, and security posture before adding it. Prefer small, focused crates and document dependencies that establish lasting architectural decisions.

## Commits

Keep commits focused and independently reviewable. Use Conventional Commits and describe the behavior change, validation performed, and relevant risks.
