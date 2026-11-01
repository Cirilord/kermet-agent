# Conventions

## Module Layout

Organize module directories in this order when the files apply:

```text
module/
├── mod.rs
├── tests.rs
├── error.rs
├── utils.rs
└── ...
```

Use `tests.rs` for unit tests, `error.rs` for module error types, and `utils.rs` only for independent helper functions. Add other files only for distinct module responsibilities.

Keep `mod.rs` free of implementation code. It may declare child modules, import items, and expose public exports or re-exports only.

Declare `#[cfg(test)] mod tests;` last in every `mod.rs`, after module declarations, imports, exports, and re-exports.

## Tests

Place a module's unit tests in `tests.rs` at the same directory level as the module being tested. Declare that file from the module with `#[cfg(test)] mod tests;`. Do not place unit tests in production source files.

## Imports

Prefer `super` for items in the parent module or its descendants. Use `crate` only when the item is outside that local module boundary or when an absolute path materially improves clarity.

## Derives

Derive only the traits required by the current implementation, public interface, or tests. Do not add speculative `derive` directives.

## Visibility

Use the narrowest visibility that satisfies the caller. Prefer private items, then `pub(super)` or `pub(in ...)`, and use `pub(crate)` only when a crate-level boundary requires it. Use `pub` only for an intended public library interface.

## Errors

Define a module's error types in its `error.rs` file. Use a single `error.rs` rather than an `errors/` directory unless separate error modules become necessary.
