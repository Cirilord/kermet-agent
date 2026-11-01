# Product Direction

## Philosophy

Kermet uses deterministic systems whenever they can reliably perform work. LLM calls are reserved for judgment, ambiguity resolution, and generation.

Before invoking an LLM, use deterministic capabilities to narrow the task, prepare structured context, and validate results. The model should receive the smallest useful context needed for the current task.

## Product Scope

Kermet is a local-first coding agent. Its goal is to inspect repositories, make scoped changes, and validate results within explicit execution boundaries.

The product prioritizes reliable task completion, clear control over local access, and efficient use of model context over the number of available features.

## Delivery Principles

- Build the smallest complete capability before expanding scope.
- Prefer deterministic validation over model judgment when possible.
- Make security a release requirement for every tool and integration.
- Preserve observable state and evidence for actions that affect a workspace.
