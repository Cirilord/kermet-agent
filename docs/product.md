# Product Direction

## Philosophy

Kermet uses deterministic systems wherever they can reliably do the work. LLM calls are reserved for tasks that require judgment, ambiguity resolution, or generation.

Before invoking an LLM, use deterministic capabilities to narrow the task and validate its result. When designing a capability, ask:

1. Can this be solved reliably with deterministic code?
2. Can deterministic code narrow the task or validate the result before or after an LLM call?
3. Does the LLM receive the smallest useful, well-structured context?

Use an LLM only when its probabilistic reasoning adds value beyond those deterministic steps.

## Product scope

Kermet is a local-first coding agent. Its initial goal is to investigate a repository, make scoped changes, and validate them autonomously within explicit execution boundaries.

The differentiator is not more tools. It is using structured, deterministic context retrieval and evidence-based validation to complete work with less waste.

Security is a release gate for every tool and integration; see `docs/security.md`.

## Delivery order

1. Agent loop, model provider interface, file tools, repository search, terminal execution, and permissions.
2. Persistent sessions, token-aware context compaction, structured task state, and validation evidence.
3. File and symbol indexes, dependency graph, incremental updates, and targeted context retrieval.
4. Semantic retrieval with embeddings and vector search.
5. MCP, skills, subagents, and adaptive model routing when the core has measurable baselines.
