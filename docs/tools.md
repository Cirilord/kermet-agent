# Agent Tools

The agent loop accepts only typed requests validated and executed through `ToolRegistry`. A model can request a declared tool, but cannot bypass authorization or access data outside the tool boundary.

| Tool | Requirement | Status | Input | Authorization and validation | Output to the model |
| --- | --- | --- | --- | --- | --- |
| `bash` | `danger-full-access`, network allowed | Available without process sandbox | A non-empty Bash command of at most 16 KiB | Runs with the workspace as its working directory, no standard input, a cleared environment except `PATH`, a 30-second timeout, and bounded stdout and stderr. It is not filesystem- or network-isolated until the sandbox boundary is implemented. | Exit code, timeout state, bounded stdout and stderr, and truncation flags, encoded as JSON. |
| `read` | `read-only`, network denied | Available | Workspace-relative or absolute path; optional inclusive `start_line` and `end_line` | The path is canonicalized, must remain within the workspace after symlink resolution, must identify a regular file, and is limited to 256 KiB. Line ranges begin at 1 and must have `end_line >= start_line`. | Canonical path and requested file contents, encoded as JSON. |
| `write` | `workspace-write`, network denied | Available | Workspace-relative or absolute path and complete UTF-8 content | The parent path and an existing target are canonicalized and must remain within the workspace after symlink resolution. The target is a regular file or a new path in an existing directory; content is limited to 256 KiB. The tool writes a temporary sibling file, syncs it, and replaces the target with a rename. | Canonical target path and bytes written, encoded as JSON. |
| `edit` | `workspace-write`, network denied | Available | Workspace-relative or absolute path, exact `old_text`, and replacement `new_text` | The target is canonicalized and must be a UTF-8 regular file within the workspace after symlink resolution. `old_text` must be non-empty and occur exactly once. The resulting content is limited to 256 KiB and is written atomically through a synced sibling temporary file. | Canonical target path and bytes written, encoded as JSON. |

## Registration flow

1. The provider validates an external function call against the declared schema and creates a typed `ToolRequest`.
2. `ToolRegistry` dispatches the request to the matching tool.
3. The tool declares its `ExecutionRequirement`; `AuthorizationManager` authorizes it against the active `ExecutionPolicy` before input validation and execution.
4. The registry serializes the bounded result for the provider continuation request.

Tools must be added to the provider schema, typed request, registry, execution requirement, security tests, and this table in the same change.
