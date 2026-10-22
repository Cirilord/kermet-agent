# Configuration

Kermet reads non-secret settings from `~/.kermet/config.toml`. Credentials never belong in this file; provider credentials are stored in the operating system credential store.

Create the file and manage its supported keys with:

```sh
kermet config init
kermet config set profiles.default.provider codex
kermet config set profiles.default.model gpt-6.1-sol
kermet config set profiles.default.reasoning_effort medium
kermet config get profiles.default.model
kermet config check
```

`config unset profiles.default.model` removes a value. `config check --profile NAME` resolves and validates the selected profile. The commands accept only `provider`, `model`, and `reasoning_effort` keys under `profiles.<name>`.

## Model profiles

Each profile selects a provider, model, and reasoning effort.

```toml
[profiles.default]
provider = "codex"
model = "gpt-6.1-sol"
reasoning_effort = "medium"
```

Run a task with the selected profile:

```sh
kermet run "Inspect src/main.rs"
```

Use `--profile` to select another configured profile. Per-run flags override individual profile values:

```sh
kermet run \
  --model gpt-6-luna \
  --reasoning-effort high \
  "Inspect src/main.rs"
```

## Interactive threads

Start an interactive thread with `run --interactive` (or `run -i`):

```sh
kermet run -i
```

Kermet creates a thread, records the workspace and model profile selected when it was created, and prints its identifier. Enter messages until `/exit` or end-of-file closes the session. When resuming, Kermet renders the saved transcript in the same terminal format before showing the next prompt. Resume it later with:

```sh
kermet run -i --resume thread-0123456789abcdef
```

Threads are stored as versioned JSON documents in `~/.kermet/threads`. They contain the workspace path, profile name, and user and assistant messages, so do not enter secrets in a thread. Credentials remain in the operating system credential store.

This initial implementation sends the recorded conversation history with each new turn. Token-aware context selection and compaction will replace that behavior in a later iteration.

## Resolution

Kermet resolves provider settings in this order:

1. `run` command-line flags;
2. the selected profile in `~/.kermet/config.toml`;
3. an explicit configuration error.

Kermet does not select a provider, model, or reasoning effort implicitly. `config check` validates the local configuration; provider-side model availability is validated when the provider executes a request.
