# `settings` Crate — Configuration & Live Deltas

A focused, type-safe Settings system for Forge of Stories (and reusable elsewhere):

* Strongly-typed sections (`Settings` trait, `const SECTION`)
* In‑memory merged view = `defaults < user_deltas`
* Persistent file stores ONLY the delta vs defaults (RON)
* Recursive (nested struct) diffing
* Optional filesystem watcher (hot‑reload)
* Optional Bevy integration (resources auto-updated)
* Lightweight macro (`define_settings!`) for ergonomic section setup
* Path-aware error diagnostics

---

## Contents

1. Quick Start (manual struct)
2. Using the `define_settings!` macro
3. Delta File Semantics
4. API Reference (`SettingsStore`)
5. Live Reload (`reload`, watcher feature)
6. Error Model
7. Bevy Integration
8. Concurrency Notes
9. FAQ
10. Roadmap / Future Extensions

---

## 1. Quick Start (manual struct definition)

```rust
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use settings::{Settings, SettingsStore, SettingsError};

#[derive(Clone, Serialize, Deserialize)]
struct Network {
    port: u16,
    label: String,
}
impl Default for Network {
    fn default() -> Self {
        Self { port: 100, label: "main".into() }
    }
}
impl Settings for Network {
    const SECTION: &'static str = "network";
}

fn main() -> Result<(), SettingsError> {
    // 1) Store aufbauen – Datei enthält nur Abweichungen
    let store = SettingsStore::builder()
        .with_settings_file(paths::config_dir().join("settings.ron"))
        .build()?;

    // 2) Section registrieren (lädt Default + wendet Delta an)
    store.register::<Network>()?;

    // 3) Lesen (Snapshot; Arc für Billig-Clones)
    let net: Arc<Network> = store.get::<Network>()?;
    assert_eq!(net.port, 100);

    // 4) Aktualisieren (nur Diff zu Default landet in Datei)
    store.update::<Network, _>(|n| n.port = 4242)?;

    Ok(())
}
```

---

## 2. (Macro usage removed)

The earlier macro-based convenience (`define_settings!`) is currently not part of the supported workflow.
Define your settings structs manually (derive `Serialize`, `Deserialize`, `Clone`, implement `Default` + `Settings`).

If you already have struct definitions: use `define_settings!` to auto-derive `Default` + `Settings`.

```rust
use serde::{Serialize, Deserialize};
use settings::{define_settings, SettingsStore};

#[derive(Clone, Serialize, Deserialize)]
pub struct Limits {
    pub enabled: bool,
    pub level: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Network {
    pub port: u16,
    pub limits: Limits,
}

define_settings! {
    network: Network {
        port: 100,
        limits: Limits { enabled: true, level: 1 },
    };
}

fn main() {
    let store = SettingsStore::builder()
        .with_settings_file(paths::config_dir().join("settings.ron"))
        .build()
        .unwrap();

    store.register::<Network>().unwrap();
    store.update::<Network, _>(|n| n.port = 7777).unwrap();
}
```

Resulting delta file (`settings.ron`) after the update:

```/dev/null/settings.ron#L1-12
{
    "network": {
        "port": 7777,
    },
}
```

Note:
* Unchanged nested defaults (e.g. `limits.enabled`, `limits.level`) are omitted.
* A nested change would create nested diffs, e.g.:

```/dev/null/settings_nested_delta.ron#L1-18
{
    "network": {
        "limits": {
            "level": 5,
        },
        "port": 7777,
    },
}
```

---

## 3. Delta File Semantics

Principle: Persist only what *differs* from the default tree (recursive).

Algorithm (for a section):
1. Serialize current instance (map).
2. Compare recursively to default map.
3. Keep only keys where value != default (recurse for nested maps).
4. Empty diff → remove section from delta file.

Implications:
* Adding a new field with a default value → not written until changed.
* Removing a field from code: stale entry in delta file is ignored if default no longer has it (not auto-pruned yet).

---

## 4. API Reference (Essentials)

```/dev/null/api_summary.rs#L1-120
// Build
let store = SettingsStore::builder()
    .with_settings_file(path)
    .build()?;

// Register (must be called before get/update)
store.register::<MySection>()?;

// Get snapshot
let arc_cfg = store.get::<MySection>()?;      // Err if not registered
let maybe = store.try_get::<MySection>()?;    // Ok(None) if not registered

// Update (closure mutates a temporary instance)
store.update::<MySection, _>(|s| {
    s.enabled = true;
    s.tuning.factor = 2.5;
})?;

// Reload (re-read file, re-apply all registered sections)
store.reload()?;

// File path
let p = store.file_path();

// Thread-safety: internal RwLock; get/update can be called from multiple threads.
```

Important rules:
* `register::<T>()` must precede `get` / `update`.
* Multiple registrations of same type → error (`Invalid("section already registered")`).
* `get` returns a snapshot (mutations require `update`).
* `update` merges diff & persists atomically (temp file + rename).
* Atomic semantics rely on filesystem rename guarantees (POSIX-like).

---

## 5. Live Reload (manual only)

### Manual Reload
Call:
```/dev/null/reload.rs#L1-10
store.reload()?; // Re-parse delta file & re-merge all registered sections
```

### (File watcher removed)
Previous optional filesystem watcher support has been dropped for now to keep the system minimal.
```
settings = { version = "...", features = ["watch"] }
```

Usage:
```/dev/null/watcher.rs#L1-40
use std::sync::Arc;
#[cfg(feature = "watch")]
use settings::start_settings_watcher;

let store = Arc::new(SettingsStore::builder()
    .with_settings_file(paths::config_dir().join("settings.ron"))
    .build()?);

store.register::<Network>()?;

#[cfg(feature = "watch")]
let watcher = start_settings_watcher(store.clone())?;

// Keep `watcher` in scope (drop stops watching).
```

To pick up external edits call `store.reload()` at suitable points in your application loop (e.g. on a debounced timer).

---

## 6. Error Model

`SettingsError` variants:

| Variant                  | Meaning |
|--------------------------|---------|
| `Io(std::io::Error)`     | File system issues (read/write/rename) |
| `Ron(ron::Error)`        | Serialization / Deserialization failure |
| `Invalid(&'static str)`  | Logic / invariant violation (internal) |
| `NotRegistered`          | Access to unregistered section |
| `Path { path, msg }`     | Contextual parse / conversion error with file path |

Examples:
* Corrupt file → `Path { path: ".../settings.ron", msg: "parse settings file" }`
* Attempt double register → `Invalid("section already registered")`
* Get before register → `NotRegistered`

Use `match` or `Display` to format; path-aware variant aids UI surfacing.

---

## 7. Bevy Integration (Feature `bevy`)

Feature:
```
settings = { version = "...", features = ["bevy"] }
```

### Registering Sections as Resources

```/dev/null/bevy_integration.rs#L1-120
use bevy::prelude::*;
use settings::{SettingsStore, Settings, SettingsError};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Gameplay {
    difficulty: u8,
}
impl Default for Gameplay { fn default() -> Self { Self { difficulty: 1 } } }
impl Settings for Gameplay { const SECTION: &'static str = "gameplay"; }

fn main() -> Result<(), SettingsError> {
    let store = SettingsStore::builder()
        .with_settings_file(paths::config_dir().join("settings.ron"))
        .build()?;
    store.register::<Gameplay>()?;

    App::new()
        .insert_settings_store(store)      // inserts `SettingsStoreRef`
        .register_settings_section::<Gameplay>() // inserts `SettingsArc<Gameplay>`
        .add_systems(Update, print_difficulty)
        .run();
    Ok(())
}

fn print_difficulty(res: Res<settings::SettingsArc<Gameplay>>) {
    // Updated automatically if watcher + reload integrated externally
    println!("Difficulty = {}", res.0.difficulty);
}
```

### Hot Reload Strategy

You control the reload trigger (e.g. watcher event sets a channel → system calls `store.reload()` → system compares & updates resources). The provided adapter already updates the Bevy resource if a changed `Arc` is returned by `get::<T>()`.

---

## 8. Concurrency Notes

### Logging Hook

You can install a logger to route internal messages (reload errors, etc.) away from `eprintln!`:

```/dev/null/logging_hook.rs#L1-30
use settings::{SettingsStore, Settings, SettingsError};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Demo { value: u32 }
impl Default for Demo { fn default() -> Self { Self { value: 1 } } }
impl Settings for Demo { const SECTION: &'static str = "demo"; }

fn main() -> Result<(), SettingsError> {
    let store = SettingsStore::builder()
        .with_settings_file(std::env::temp_dir().join("demo_settings.ron"))
        .with_logger(|msg| println!("[settings] {msg}"))   // <— custom logger
        .build()?;

    store.register::<Demo>()?;
    store.update::<Demo, _>(|d| d.value = 42)?;
    Ok(())
}
```

You can also replace it later:
```/dev/null/set_logger.rs#L1-10
store.set_logger(|msg| tracing::info!("settings: {msg}"));
```

### Pruning Stale Deltas

When you rename / remove fields or whole sections, old keys may linger in the delta file.
Call `prune_stale()` to remove:
* Sections not (anymore) registered.
* Keys that no longer exist in the current defaults (recursively).
* Entire section entries that become empty after pruning.

```/dev/null/prune.rs#L1-20
// After refactors or before shipping a cleaner file:
store.prune_stale()?;   // Writes updated (reduced) delta file
```

The pruning operation:
1. Clones defaults snapshot
2. Walks each delta section
3. Drops unknown keys (depth-first)
4. Persists only if changes occurred (internally via `persist_deltas()`)

Performance: O(size_of_delta); safe to run occasionally (e.g. on a maintenance command).


* Internally: `RwLock` around maps (defaults, deltas, merged values).
* `get` performs (serialize → parse) roundtrip from cached `ron::Value`. For high-frequency reads you can cache `Arc<T>` elsewhere.
* `update`:
  1. Deserialize current (`O(size_of_struct)`).
  2. Mutate closure.
  3. Serialize & diff (recursive map walk).
  4. Write delta file if changed.
* Many small rapid updates → last-writer wins; no built-in batching (roadmap possibility).

---

## 9. FAQ

**Q: Why RON instead of TOML/JSON/YAML?**
RON integrates with `serde`, supports richer data (enums) cleanly, and keeps diffs compact.

**Q: Can I store enums / options?**
Yes; they serialize through `serde` and will diff like any other value.

**Q: What about removing obsolete keys?**
Currently they remain until rewritten; they are ignored if not present in defaults. A cleanup pass can be added later.

**Q: Deeply nested structures?**
Recursive diff already supports arbitrarily nested maps (structs). Non-map types (arrays) are compared for equality wholesale.

**Q: Live reactive updates without polling?**
Use watcher feature + call `reload()`. Integrate in Bevy via a system.

---

## 10. Roadmap / Future Extensions

| Item | Status | Notes |
|------|--------|-------|
| Inline struct + defaults macro (single macro without pre-decl structs) | Planned | Prototype removed pending simplified version |
| Logging hook (replace `eprintln!`) | Planned | Provide pluggable logger or `log` crate adapter |
| Prune stale keys on reload | Planned | Optional hygiene step |
| Batch updates / transaction API | Planned | To reduce repeated serialization |
| Validation hook (post-mutate) | Planned | `update_with_validation(|s| { ... }, |old,new| Result)` |
| Inline macro nested depth > 1 | Planned | Needs stable recursion design |
| Serde schema export | Future | For UI generation / docs |
| Partial reload per section | Future | Optimization |

---

## Minimal Cheat Sheet

```/dev/null/cheatsheet.rs#L1-70
// Define
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Audio { volume: f32 }
impl Default for Audio { fn default() -> Self { Self { volume: 0.8 } } }
impl settings::Settings for Audio { const SECTION: &'static str = "audio"; }

// Build store
let store = settings::SettingsStore::builder()
    .with_settings_file(paths::config_dir().join("settings.ron"))
    .build()?;

// Register
store.register::<Audio>()?;

// Read
let a = store.get::<Audio>()?;
println!("Vol = {}", a.volume);

// Update
store.update::<Audio, _>(|cfg| cfg.volume = 0.9)?;

// Try get (optional)
if let Some(a2) = store.try_get::<Audio>()? {
    println!("Updated Vol = {}", a2.volume);
}

// Reload after external edit
store.reload()?;

// (Feature "watch")
#[cfg(feature = "watch")]
let _watcher = settings::start_settings_watcher(std::sync::Arc::new(store));
```

---

## License

Part of the Forge of Stories workspace. See root repository license (all workspace crates follow same terms unless stated otherwise).

---

Happy configuring! Contributions & refinements welcome—open issues for feature proposals or macro improvements.
