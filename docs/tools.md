# Agent Tools

The agent loop accepts only typed requests validated and executed through `ToolRegistry`. A model can request a declared tool, but cannot select a capability, bypass authorization, or access data outside the tool boundary.

| Tool | Capability | Status | Input | Authorization and validation | Output to the model |
| --- | --- | --- | --- | --- | --- |
| `read_file` | `read-file` | Available | Workspace-relative or absolute path; optional inclusive `start_line` and `end_line` | Allowed in `read-only` and `workspace-write`. The path is canonicalized, must remain within the workspace after symlink resolution, must identify a regular file, and is limited to 256 KiB. Line ranges begin at 1 and must have `end_line >= start_line`. | Canonical path and requested file contents, encoded as JSON. |

## Registration flow

1. The provider validates an external function call against the declared schema and creates a typed `ToolRequest`.
2. `ToolRegistry` dispatches the request to the matching tool.
3. The tool authorizes the capability through `PermissionManager` and validates its input before executing.
4. The registry serializes the bounded result for the provider continuation request.

Tools must be added to the provider schema, typed request, registry, permission policy, security tests, and this table in the same change.
