# Kermet

Kermet is a local-first coding agent written in Rust.

> Less searching. More building.

It is being built around deterministic execution, explicit security boundaries, and efficient model context. The agent should use deterministic code whenever it can reliably inspect, validate, or narrow a task; language models are reserved for judgment, ambiguity resolution, and generation.

## Status

Kermet is under active development. The current codebase contains only the initial Rust executable while the final architecture is rebuilt.

## Principles

- Keep the agent core independent from interfaces, providers, tools, and platform integrations.
- Validate untrusted input at every boundary.
- Default to deny and grant the smallest access needed for an operation.
- Prefer typed, deterministic systems over implicit behavior.
- Send the smallest useful context to a model.

## Requirements

- Current stable Rust toolchain.
- Clippy and rustfmt Rust components.
- Lefthook for local Git hooks.
- Cocogitto for Conventional Commit validation.

## Run

```sh
cargo run
```

## Validation

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Documentation

- [Architecture](docs/architecture.md)
- [Conventions](docs/conventions.md)
- [Development workflow](docs/development.md)
- [Product direction](docs/product.md)
- [Security policy](docs/security.md)
