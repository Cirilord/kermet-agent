# Architecture

## Runtime

- Language and runtime: stable Rust.
- Distribution: a native executable for each supported platform.
- Design: keep the agent core independent from the CLI, model providers, filesystem, shell, network, and external integrations.

## Boundaries

The CLI parses user input and converts it into typed domain requests. It does not contain agent behavior, provider logic, security decisions, or tool execution.

The CLI returns typed actions without invoking application or provider code. The composition root receives those actions and will connect them to use cases when those use cases exist.

Provider adapters own external authentication and model protocols. The composition root parses a provider kind and converts it into a provider, but provider modules must not depend on CLI types. Providers expose shared operations directly; the current operation set contains only login.

The agent core coordinates a task through explicit state and typed interfaces. Providers translate external model protocols into internal requests. Tools validate and execute those requests through deterministic security boundaries.

External inputs, model output, tool output, persisted data, and network responses are untrusted data. They must be validated before crossing into a domain boundary.

## Modules

Each module has one clear responsibility and exposes the smallest public interface needed by its parent or consumers. Keep implementation details private and place cross-cutting decisions at explicit boundaries rather than relying on implicit global state.

## Dependencies

Dependencies must point toward the domain core. The domain core must not depend directly on interface adapters such as the CLI, providers, or platform integrations.
