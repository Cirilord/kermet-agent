# CLI Commands

| Command | Arguments | Flags | Description | Status |
| --- | --- | --- | --- | --- |
| `kermet config get <key>` | Configuration key path. | None | Prints a string configuration value. Paths may address TOML tables, such as `profiles.default.model`. | Available |
| `kermet config set <key> <value>` | Configuration key path and string value. | None | Sets a string configuration value, creating missing TOML tables in the path. | Available |
| `kermet config show` | None. | None | Prints `~/.kermet/config.toml`. Kermet creates the file when it is missing. | Available |
| `kermet config unset <key>` | Configuration key path. | None | Removes a configuration value. Paths may address TOML tables. | Available |
| `kermet login <provider>` | Provider identifier. | None | Signs in with the selected provider. `codex` is currently supported. | Available |
| `kermet models` | None. | None | Lists the models available to the active provider. | Available |
| `kermet logout` | None. | None | Revokes the active provider's renewable session when possible and clears its local tokens. `codex` is currently supported. | Available |
