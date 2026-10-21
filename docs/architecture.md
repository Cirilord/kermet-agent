# Architecture

## Runtime

- **Language and runtime:** Rust.
- **Distribution:** a native executable for each supported platform.
- **Design:** the agent core must remain independent from the CLI, model providers, filesystem, shell, and external integrations.

## CLI boundary

The CLI parses user input and validates the workspace before constructing a typed request for the runtime. It does not contain agent behavior, provider logic, permission decisions, or tool execution.

The initial commands are `doctor`, `run`, `login`, `logout`, and `config`. `doctor` reports deterministic local state. `login` coordinates provider authentication and writes credentials through the credential-store boundary; `login status` reads that boundary without exposing secrets. `logout` removes the selected credential profile. `config` manages non-secret model profiles. `run` resolves a typed provider, model, and reasoning-effort selection before constructing the runtime request; it fails when any selection value is missing.

## Task request

`TaskRequest` is the typed boundary between the CLI and agent core. It contains a non-empty task, a validated workspace, and the requested permission mode. It carries domain data only: Clap types, provider configuration, and tool execution stay outside it.

## Core components

| Component | Responsibility |
| --- | --- |
| Agent core | Runs an explicit, observable state machine for a task. |
| Model provider | Translates provider-specific model and tool-calling protocols into internal types. |
| Tool registry | Defines, validates, and executes available tools. |
| Permission manager | Applies deterministic authorization policy before tool execution. |
| Context manager | Selects the smallest useful context under a token budget. |
| Codebase context | Indexes and retrieves files, symbols, relationships, and cached code ranges. |
| Evidence store | Records changes, commands, test results, and validation outcomes. |
| Credential store | Persists provider credentials through an explicit secure-storage boundary. |

Tools, permissions, validation, and providers must not be implicit dependencies of the agent loop.

## Credentials

Provider adapters exchange provider-specific token payloads through the credential-store interface. Persistent credentials use the operating system credential store, keyed by provider and profile. An explicit ephemeral store supports tests, CI, and short-lived processes. Credential storage is independent of the workspace and agent-session stores.

## Codebase retrieval

Retrieval is progressive. Start with local, deterministic operations and only send selected results to the model:

1. Discover files with glob and `rg --files`, respecting ignore rules.
2. Search paths and contents with glob and ripgrep.
3. Read bounded ranges instead of whole files whenever possible.
4. Retrieve symbols and references when a language integration supports them.
5. Expand from relevant symbols through dependency relationships.

Each indexed file records its path, language, size, modification metadata, and content hash. Hashes are used to invalidate indexes and cached ranges incrementally.

## Symbol index and language support

The index normalizes language-specific results into common concepts: files, modules, symbols, definitions, references, imports, and dependencies.

- Start with Tree-sitter for syntax and symbol extraction.
- Use Language Server Protocol integrations to enrich definitions and references when available.
- Add language-specific adapters only when Tree-sitter and LSP cannot provide the required relationship.
- Keep unresolved and dynamic dependencies explicit; missing graph edges are never proof that no relationship exists.

## Dependency graph

The graph is a local code knowledge graph, not a separate graph-database service.

- **Storage:** SQLite.
- **Initial nodes:** files, modules, symbols, and tests.
- **Initial edges:** `IMPORTS`, `DEFINES`, `REFERENCES`, `CALLS`, `TESTS`, and `BELONGS_TO` where they can be derived reliably.
- **Edge metadata:** origin, resolution status, and confidence.

Start with file imports and symbol definitions. Add call, reference, and test relationships only when their extraction is reliable enough for the language.

## Vector search

Vector search is a later retrieval layer for vague, semantic questions that exact search and symbol lookup cannot answer well.

- Generate embeddings for content-addressed code chunks.
- Update only chunks whose hash changed.
- Keep embeddings alongside the local index, initially using SQLite with a vector extension if Rust integration is suitable; otherwise use LanceDB.
- Combine vector candidates with text matches, symbols, and graph neighbors before selecting context.

Vector search does not replace ripgrep, symbol lookup, or the dependency graph.

## State, memory, and validation

- Persist full session history, but treat it separately from the model context.
- Compact conversation history into structured task state: decisions, files changed, evidence, failed attempts, and pending work.
- Store durable project knowledge with source, scope, verification time, and invalidation conditions.
- Detect repeated actions and unchanged failures deterministically, then change strategy or request more context.
- Do not declare a task complete without recorded evidence such as relevant tests, static analysis, command results, or explicit unmet criteria.
