# Plan: SettingsStore → TOML, Game-Oriented

Derived solely from crates/settings/memories/settings_store_rs_analysis.md.

## Extracted Model (from memories)
- Hierarchical settings with many layers (default, global, extensions, user, release, OS, profile, server, project stack).
- Settings trait with KEY/FALLBACK_KEY/PRESERVED_KEYS and FileContent; load() merges sources.
- Type-erased registry of setting types; async file update queue; atomic writes.
- JSON-centric raw storage and schema generation; EditorConfig integration; selective recompute.

## TOML-Centric Redesign
- Replace raw_* JSON Values with TOML documents or typed structures:
  - raw_default_toml, raw_user_toml, raw_server_toml, etc. (as `toml_edit::Document`).
  - Keep ability to hold unknown keys for forward compatibility.
- Settings::FileContent remains typed; serde over TOML.
- Remove JSON Schema generation; validation moves to per-setting validators or a lightweight checker.

## Layering for the Game
- Simplify layers initially: default, os, release, profile, server, user, project (stacked), global/extension optional.
- Clear precedence: default < os < release < profile < server < user < project (nearest last).
- Preserve project stack semantics and EditorConfig support if needed (text files, e.g., linting).

## Update Pipeline (TOML)
- Provide `update_settings_toml<T: Settings>(store, fs, update)` using a background queue and atomic writes.
- Use `toml_edit` to apply minimal changes while preserving comments and formatting.
- Write to the highest-priority writable layer (usually user; project when local editing).

## Type Registry (TOML)
- Keep AnySettingValue with TOML-aware methods:
  - deserialize_setting_toml(&self, doc: &toml_edit::Document) -> Result<DeserializedSetting>.
  - edits_for_update_toml(...): compute edits for minimal-diff updates.

## SettingsSources<T>
- Keep structure but change serialization type from JSON to typed T loaded from TOML.
- Allow FALLBACK_KEY by probing alternate table paths in TOML.
- PRESERVED_KEYS honored by always writing specified keys even when equal to defaults.

## Async & Performance
- Maintain mpsc queue and background task model; use spawn_blocking for TOML parse/serialize.
- Selective recompute on changes; reuse BTreeMap for local path hierarchy.

## Migration & Compatibility
- One-time import from legacy JSON on first run (if files exist), then rewrite to TOML.
- Provide debug logging for layer sources and effective value resolution.

## Open Questions
- Keep full EditorConfig integration for the game, or defer until text-editing features need it?
- Which layers are writable from the TUI vs. server admin (e.g., user vs. server)?
- Do we expose a public schema (TOML) or rely on docs + examples?

## Implementation Steps
1) Introduce TOML raw storage in SettingsStore and parsing helpers.
2) Implement `update_settings_toml<T>` with background queue and atomic writes.
3) Port SettingsSources and load() to TOML-backed data; define precedence.
4) Add per-setting validators and minimal-diff edit computation using toml_edit.
5) Add migration helper from JSON → TOML; add tests for recompute and layering.
