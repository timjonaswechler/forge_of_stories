# Design Document: Forge of Stories – Settings System (TOML-first, Updated)

This document captures the architecture, current implementation status, and the roadmap for the Forge of Stories settings system. It is kept up to date as we build out the core, server integration, and client integration.

Contents:
- 1) Purpose and Scope
- 2) High-Level Goals
- 3) Current Status (What’s done)
- 4) Architecture & Implementation (TOML-first)
- 5) File Layout and Layering (TOML)
- 6) Server Configuration Integration (Diff policy, live reload)
- 7) Migrations (Registry; current stance)
- 8) Hot-Reload (Watcher)
- 9) Input Mapping (Planned)
- 10) Risks and Mitigations
- 11) Roadmap (Next steps)

---

1) Purpose and Scope
- Provide a robust, reusable settings system for both Client (Bevy) and Dedicated Server (headless) in Rust.
- TOML-first format with preserved comments and formatting on edits.
- Strong layering model (global, user, per-save/world, server).
- Hot-reload support and a clear policy for mutable vs. immutable changes (especially on the server).

Scope excludes gameplay/server runtime logic; focuses on settings representation, IO, diffs, and APIs.

---

2) High-Level Goals
- Cross-platform: Windows, macOS, Linux.
- Pure Rust; no GUI coupling in the core.
- TOML-first: human-friendly configs, comments preserved.
- Layered settings with predictable merge semantics.
- Hot-reload; server marks immutable changes as restart-required.
- Central write API to preserve formatting/comments for Admin CLI/Web.
- Optional validation/documentation via schemas (schemars) for future UIs.

Note: The original JSON-centric plan has been replaced by TOML-first across the board.

---

3) Current Status (What’s done)

Core (crate: `crates/setting_core`)
- Format-agnostic core API with TOML backend:
  - ConfigBackend trait (TOML implementation provided).
  - SettingsStore (load, effective values via layering, update, watch).
  - DeepMerge implemented for TOML values (tables merge, other types replace).
- TOML backend (feature: `backend-toml`):
  - parse: text → `toml_edit::Document` (preserves comments/formatting).
  - to_value/from_value: `Document` ↔ `toml::Value`.
  - deserialize<T>: `toml::Value` → T (serde).
  - empty_root: `{}` for starting layer merges.
  - dom_to_string: `Document` → String (for persistence).
  - root_version: reads top-level `version` (if present).
  - apply_updates:
    - Set: Key-only paths (create intermediate tables).
    - Remove: supports removing table keys and array elements (Index/Match).
    - UpsertInArray: supports arrays-of-tables (inline or ArrayOfTables); match by key=value.
- Watcher (feature: `watch`):
  - Notify-based watcher `NotifyWatcher` that watches parent directory and filters events for specific files.
- Built-ins:
  - `SimplePathResolver`: maps `FileKind` to paths under a root (settings/, saves/<id>/, server/).
  - `FsSettingsIo`: simple atomic writes (tmp + rename) and load.

Layering
- `SettingsStore::effective_from_chain` and `effective`: compute merged TOML values and deserialize into typed structs.

Migration Registry
- Implemented in core (FileClass, registry, apply steps sequentially while version is behind; persist after each step).
- Current stance: no migrations are registered or required yet.

Docs
- This document updated to TOML-first.
- JSON-specific references removed/replaced.

---

4) Architecture & Implementation (TOML-first)

Crate layout
- `setting_core` (no UI deps):
  - `ConfigBackend` (format-aware; TOML backend via `toml_edit` + `toml`).
  - `SettingsStore`: loading, merging layers, updates, optional watching.
  - `DeepMerge` trait: implemented for `toml::Value` (tables merge; arrays replaced by default).
  - `builtin` module: `SimplePathResolver`, `FsSettingsIo`.
  - `toml_backend` module (feature `backend-toml`): full TOML support as above.
  - `watch_notify` module (feature `watch`): notify-based watcher.
  - Migration registry: register per `FileClass` and version; run while version < latest.
- Future adapters:
  - settings-server (headless).
  - settings-bevy (client integration).
  - input-mapping (generalized keybindings).

Core traits/APIs (key points)
- `ConfigBackend`:
  - parse, to_value, from_value, deserialize<T>, empty_root, dom_to_string, root_version, apply_updates (Set/Remove/UpsertInArray).
- `SettingsStore`:
  - `effective`, `effective_from_chain` (layering; needs `DeepMerge` on backend value).
  - `update`, `set`, `remove`: centralized write path (preserves comments/formatting).
  - `watch`: attach a watcher and register a change callback.
  - `register_migrator`: register DOM-in-place migration steps per `FileClass` and from_version.

---

5) File Layout and Layering (TOML)

Baseline path conventions (via `SimplePathResolver`):
- settings/global.toml          → global device-wide defaults
- settings/user.toml            → user-specific overrides (optional)
- settings/keybinds.toml        → input mappings (planned)
- saves/<save_id>/world.toml    → per-save world settings
- server/server.toml            → dedicated server config

Layering model (later wins):
- Defaults (Rust `Default` on structs)
- Global
- Profiles (inside user; optional)
- User
- World (per save)
- Server (for server runtime)
- Optional: OS-specific or CLI/ENV (reserved for future)

Deep-merge semantics:
- Tables: recursive merge.
- Arrays: replace by default (programmatic array edits via UpsertInArray).

---

6) Server Configuration Integration (Diff policy, live reload)

ServerConfig (current fields):
- generic: server_managment_mode
- network: quic_port, admin_port, bind_address, max_connections
- security: crl_update_periode, cert_dir, ca_name, client_cert_name, client_key_name, crl_name
- game: game_name, game_version

Recommendations:
- Add `#[serde(default)]` on all structs for robust loading from partial TOML.
- Consider renaming `crl_update_periode` → `crl_update_period_secs` for clarity.
- Optional future fields: `version`, `save_path`, `whitelist`, `slots`.

Mutable vs. Immutable policy (proposed):
- Immutable (restart required):
  - bind_address, quic_port, admin_port
  - security: crl_update_period_secs, cert_dir, ca_name, client_cert_name, client_key_name, crl_name
  - generic: server_managment_mode
  - optional future: save_path, whitelist, slots
- Mutable (live-apply):
  - max_connections
  - game_name, game_version (if purely informational)

Flow:
- On file change:
  - Load effective ServerConfig via `SettingsStore::effective`.
  - Diff old vs. new:
    - If immutable changes present: set `restart_required`, optionally apply mutable ones; keep old immutable runtime state.
    - If only mutable: apply live and update current.
- Admin CLI/Web:
  - Use central `SettingsStore::update`/`set`/`remove` APIs to preserve file comments/formatting.

---

7) Migrations (Registry; current stance)

- Registry implemented; works per `FileClass` with sequential `from_version` steps.
- Operates directly on `toml_edit::Document` to preserve comments and formatting.
- Current stance: no migrations registered or required now (but ready for future).

---

8) Hot-Reload (Watcher)

- Notify-based watcher available behind feature `watch`.
- Watches parent directory of a file (NonRecursive), filters events for the target path.
- Server/client adapters should route callbacks into their runtime thread (e.g., via channel).

---

9) Input Mapping (Planned)

- Plan: keyboard, mouse, and gamepad support from MVP.
- Concepts:
  - Actions (jump, attack), Axes (move_x/look_y with scale/deadzone), Contexts (menu, gameplay).
  - Devices: keyboard, mouse_button, mouse_axis, gamepad_button, gamepad_axis.
  - Multiple bindings allowed; contexts resolve conflicts.
- Implementation:
  - `keybinds.toml` schema (structs), validation, deserialization.
  - Integration hooks for Bevy input systems (later).
  - Comment-preserving edits via the same TOML backend operations (Set/Remove/UpsertInArray).

---

10) Risks and Mitigations

- Comment-preserving edits complexity
  - Mitigation: Use `toml_edit::Document` for decor-preserving operations.
- Runtime apply safety (server)
  - Mitigation: Strict diff policy, restart-required for immutable changes, channel changes to main loop.
- Over-coupling to client frameworks
  - Mitigation: Keep core Bevy-free; add adapters separately.
- Migrations correctness
  - Mitigation: Registry is in place; add tests when migrations are introduced.

---

10) Roadmap (Next steps)

Completed
- [x] TOML-first design (no JSON intermediates).
- [x] Core crate `setting_core` with:
  - [x] ConfigBackend (TOML backend via `toml_edit` + `toml`).
  - [x] SettingsStore (layering `effective`/`effective_from_chain`, centralized updates).
  - [x] DeepMerge for TOML tables.
  - [x] Built-ins: `SimplePathResolver`, `FsSettingsIo`.
  - [x] Watcher (feature `watch`): notify-based.
  - [x] Update ops: Set (Key-only), Remove (Key/Index/Match), UpsertInArray (arrays-of-tables).
  - [x] Migration registry (per FileClass), `root_version`, `dom_to_string`.

In Progress / Short-term
- [ ] Server settings manager in `network/server`:
  - [ ] Add `#[serde(default)]` to ServerConfig and sub-structs.
  - [ ] Implement runtime manager: load, diff (mutable vs immutable), apply mutable, flag restart_required.
  - [ ] Provide central Admin write helpers (wrapping `SettingsStore::set/update/remove`).
- [ ] Optional: rename `crl_update_periode` → `crl_update_period_secs` and adjust everywhere.

Near-term (Core/Backend)
- [ ] Extend Set to support targeting array elements via Index/Match (optional; only if needed).
- [ ] Add example helpers for common ops (e.g., upsert mod entries in world.toml).
- [ ] Add basic tests (layering, updates, watcher) when ready.

Mid-term
- [ ] settings-bevy: Bevy plugin (Resources + Events) with hot-reload hooks.
- [ ] input-mapping: define `keybinds.toml` structs, parsing, validation.
- [ ] Documentation polishing (examples, recommended patterns).

Long-term
- [ ] Auto-generated settings UI from schemas (schemars), localization of descriptions.
- [ ] Extended profile use-cases (e.g., “Streaming”).

---

Appendix: Quick Integration Sketch (Server)
- Instantiate core:
  - `TomlBackend`, `SimplePathResolver(root)`, `FsSettingsIo`, optional `NotifyWatcher`.
- Load effective config:
  - `store.effective::<ServerConfig>(&FileKind::Server)`.
- On change:
  - Load new effective config → diff old/new → apply mutable, mark immutable as restart_required.
- Admin writes:
  - Use `store.set/update/remove` to keep comments/formatting intact.

This document will be updated as we implement the server manager, input mapping, and client integration.
