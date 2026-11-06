# CLI Commands

| Command | Arguments | Flags | Description | Status |
| --- | --- | --- | --- | --- |
| `kermet config show` | None. | None | Prints `~/.kermet/config.toml`. Kermet creates the file when it is missing. | Available |
| `kermet login <provider>` | Provider identifier. | None | Signs in with the selected provider. `codex` is currently supported. | Available |
| `kermet models` | None. | None | Lists the models available to the active provider. | Available |
| `kermet logout` | None. | None | Revokes the active provider's renewable session when possible and clears its local tokens. `codex` is currently supported. | Available |
