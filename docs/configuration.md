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

## Resolution

Kermet resolves provider settings in this order:

1. `run` command-line flags;
2. the selected profile in `~/.kermet/config.toml`;
3. an explicit configuration error.

Kermet does not select a provider, model, or reasoning effort implicitly. `config check` validates the local configuration; provider-side model availability is validated when the provider executes a request.
