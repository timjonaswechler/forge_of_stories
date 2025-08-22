# Plan: Settings File Watch/Update (TOML)

Derived solely from crates/settings/memories/settings_file_rs_analysis.md.

## Extracted Model (from memories)
- Reactive file watching for a single file or a directory (debounced ~100ms).
- Channels (unbounded mpsc) stream String contents on changes.
- Abstract filesystem via `Arc<dyn Fs>`; event kinds via `PathEventKind`.
- Convenience update API: `update_settings_file<T: Settings>(fs, cx, update)` delegates to a store.
- Test helper generates deterministic JSON defaults; platform-specific tweaks.

## Game-Oriented Changes (TOML)
- Switch on-disk format from JSON to TOML; preserve comments and formatting.
- Replace String payloads with a structured type where helpful:
  - Either `String` (raw TOML) or `Parsed<T>` for specific files.
- Keep layered writes (user > server/default); updates target the highest writable layer.
- Maintain debounce and batching semantics for responsiveness.

## API Adjustments
- Decouple from `gpui::App`/`BackgroundExecutor` by introducing an app-agnostic executor handle.
- Rename update function to `update_settings_toml<T>`; closure edits `T::FileContent` (TOML-backed).
- Perform atomic writes (temp file + replace) and return errors clearly.
- Directory watcher continues to filter by provided path set; emits create/change/remove events.

## TOML Handling
- Parse on read; if parse fails, surface error but keep streaming raw contents (partial resilience).
- For writes, use a format-preserving editor (e.g., TOML AST/DOM) to minimize diffs.
- Provide a test generator `test_settings_toml()` analogous to `test_settings()`.

## Error & Performance
- Accumulate errors while still emitting last-known-good where applicable.
- Debounce configurable (default 100ms); batch process events.
- Avoid UI blocking by running IO on a background executor.

## Open Questions
- Do we keep channel item type as `String` (raw TOML) or introduce a small enum with event kind + payload?
- Should the watcher coalesce rapid successive writes into one emission?
- Do we need per-file schema validation on read (warn-only) or strict?

## Implementation Steps
1) Define executor-agnostic watcher functions for file and dir; keep Fs trait usage.
2) Implement `update_settings_toml<T>` with atomic write and closure-based mutation.
3) Add TOML parse helpers and error reporting; keep resilience (emit even on partial failures).
4) Port the JSON test settings generator to TOML (`test_settings_toml`).
5) Document layering rules and write targets; add unit tests for watcher and updater.
