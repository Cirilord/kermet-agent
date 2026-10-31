# Architecture

## Runtime

- **Language and runtime:** Rust.
- **Distribution:** a native executable for each supported platform.
- **Design:** the agent core must remain independent from the CLI, model providers, filesystem, shell, and external integrations.

## CLI boundary

The CLI parses user input and validates the workspace before constructing a typed request for the runtime. It does not contain agent behavior, provider logic, permission decisions, or tool execution.

The initial commands are `doctor`, `hub`, `run`, `login`, `logout`, and `config`. `doctor` reports deterministic local state. `hub` serves server-rendered views of persisted threads on the loopback interface. Its workspace-picker route starts a restricted internal picker process with a hidden flag, validates and persists the selected workspace, and requires session, exact-origin, and CSRF checks. The Hub persists a new thread and its first user message, opens its Server-Sent Events stream, then starts the initial turn. Later messages use the same persisted thread and turn boundary. Hub turns currently use `danger-full-access` with network allowed to test Bash before sandboxing exists. `login` coordinates provider authentication and writes credentials through the credential-store boundary; `login status` reads that boundary without exposing secrets. `logout` removes the selected credential profile. `config` manages non-secret model profiles. `run` resolves a typed provider, model, and reasoning-effort selection before constructing the runtime request; it fails when any selection value is missing. `run --interactive` persists a thread and reuses the same typed request and agent loop for each turn.

The Hub keeps session authorization, request limits, and response protections in `src/hub/security/`. Its server module composes those boundaries with routes; server-rendered page controllers live in `src/hub/controllers/`, JSON endpoints live in `src/hub/api/`, and views do not make security decisions.

## Task request

`TaskRequest` is the typed boundary between the CLI and agent core. It contains a non-empty task, a validated workspace, and an `ExecutionPolicy`. It carries domain data only: Clap types, provider configuration, and tool execution stay outside it.

## Workspace catalog

`Workspace` is the validated runtime representation of a directory. `WorkspaceRecord` is a separate, versioned entry in the local workspace catalog, identified by a stable `workspace-` ID and containing the validated path. `WorkspaceStore` persists those records under `~/.kermet/workspaces` using the same atomic-file pattern as threads. Existing threads continue to store their workspace path until a deliberate schema migration makes them reference a workspace record.

## Core components

| Component | Responsibility |
| --- | --- |
| Agent core | Runs an explicit, observable state machine for a task. |
| Provider registry | Resolves a configured provider through its credential boundary and returns a typed provider instance. |
| Model provider | Translates provider-specific model and tool-calling protocols into internal types, preserving each accepted tool call's identifier and validated arguments for the continuation request. |
| Tool registry | Defines, validates, and executes available tools. The initial Bash tool requires `danger-full-access` with network allowed and will adopt the shared sandbox runner after platform enforcement is implemented. |
| Authorization manager | Applies deterministic execution policy before tool execution. |
| Sandbox runner | Defines the platform-independent policy, process request, resource limits, and execution contract. Its `core/` module owns those contracts, while `environments/` selects a native adapter for the current platform. macOS uses Seatbelt; unsupported platforms fail closed until a native adapter is available. |
| Context manager | Selects the smallest useful context under a token budget. |
| Codebase context | Indexes and retrieves files, symbols, relationships, and cached code ranges. |
| Evidence store | Records changes, commands, test results, and validation outcomes. |
| Credential store | Persists provider credentials through an explicit secure-storage boundary. |

Tools, permissions, validation, and providers must not be implicit dependencies of the agent loop.

## Model tool calls

The model provider translates an accepted provider function call into a typed `ToolRequest`. The tool registry validates and executes that request locally. The agent loop retains every accepted call and bounded output for the current turn, and sends that ordered history back to the provider so the model can continue the same task without repeating completed inspection. A rejected or failed tool operation returns a generic failure result to the model; it does not expose internal errors or end the whole turn before the model can explain the outcome.

Provider adapters validate external function-call payloads before creating domain requests. A provider payload cannot select an unavailable tool, add undeclared fields, or bypass the tool registry or authorization manager.

## Runtime events

Providers translate safe streaming signals into model events: the start of reasoning and final-answer text deltas. The agent loop forwards those events and emits separate events when it accepts a typed tool call and when it begins executing that tool. The CLI and Hub render these events as progress; the Hub uses a per-thread Server-Sent Events stream. Neither interface infers tool calls from unvalidated provider payloads.

Streaming text is presentation data only. The completed provider response remains the authoritative result used to decide whether a turn produced a final answer or a tool request.

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

- Internal durable resources use IDs generated and format-validated by the shared `ids` module. The resource store owns atomic collision checks when creating a record.
- Persist full session history in versioned thread documents, but treat it separately from the model context. The initial interactive implementation sends full history; it is a temporary implementation until compaction exists.
- Compact conversation history into structured task state: decisions, files changed, evidence, failed attempts, and pending work.
- Store durable project knowledge with source, scope, verification time, and invalidation conditions.
- Detect repeated actions and unchanged failures deterministically, then change strategy or request more context.
- Do not declare a task complete without recorded evidence such as relevant tests, static analysis, command results, or explicit unmet criteria.
