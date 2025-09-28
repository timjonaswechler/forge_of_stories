# `settings` crate — typed configuration with delta persistence

Forge of Stories keeps its runtime settings in one JSON delta file per product. This crate wraps that file in a thread-safe store that merges defaults with user overrides, performs schema migrations, and lets systems opt into live reloads when needed.

## What you get
- Strongly typed sections (`Settings` trait + `Default` implementation)
- Delta persistence (`defaults` overlaid with stored differences only)
- Schema version tracking and migration hooks per section
- Safe concurrent reads/updates (`RwLock` protected store, `Arc` snapshots)
- Optional logging integration that reuses the workspace tracing subscriber

## Quick start
```rust
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsError, SettingsStore};

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
    // 1) Build the store pointing at the shared delta file
    let store = SettingsStore::builder("0.2.0")
        .with_settings_file(paths::config_dir().join("settings.json"))
        .build()?;

    // 2) Register a section; defaults are merged with any stored delta
    store.register::<Network>()?;

    // 3) Read the current snapshot (cheap to clone via Arc internally)
    let net = store.get::<Network>()?;
    assert_eq!(net.port, 100);

    // 4) Update fields; only the diff vs defaults is written to disk
    store.update::<Network, _>(|cfg| cfg.port = 4242)?;
    Ok(())
}
```

## Managing multiple sections
Register each section once during startup. Repeated `register::<T>()` calls will error to protect against double initialization. Use `try_get::<T>()` when the section is optional and you want `Option<Arc<T>>` instead of an error.

```rust
store.register::<Network>()?;
store.register::<Audio>()?;

if let Some(audio) = store.try_get::<Audio>()? {
    println!("Volume {}", audio.volume);
}
```

## Schema versions & migrations
- `SettingsStore::builder("X.Y.Z")` records the current schema target for all sections.
- Each section receives the previous on-disk version in its `Settings::migrate` hook so it can upgrade data before the merged value is stored.
- Call `store.schema_version()` to inspect the active version or test the current file metadata.
- Use `store.reload()` to re-read the delta file after an external edit (e.g. CLI, admin UI).

```rust
impl Settings for Network {
    const SECTION: &'static str = "network";

    fn migrate(
        file_version: Option<&semver::Version>,
        target_version: &semver::Version,
        data: serde_json::Value,
    ) -> Result<(serde_json::Value, bool), SettingsError> {
        // inspect file_version < target_version, adjust fields, return (value, changed)
        Ok((data, false))
    }
}
```

## Logging integration
The store uses the workspace tracing subscriber when present. You can install a custom hook either during builder setup or at runtime:

```rust
let store = SettingsStore::builder("0.2.0")
    .with_settings_file(paths::config_dir().join("settings.json"))
    .with_logger(|msg| tracing::info!(target: "settings", "{msg}"))
    .build()?;

// Later, swap logger if needed
store.set_logger(|msg| println!("[settings] {msg}"));
```

If no logger is provided and tracing is not initialised, messages fall back to `eprintln!` for visibility in tests and CLIs.

## Keeping the delta file tidy
- `store.prune_stale()` removes keys that no longer appear in the defaults of registered sections.
- `store.ensure_file_version()` runs automatically to bump the `__meta.version` field when migrations succeed.
- When you rename or remove a section, call `prune_stale()` after re-registering to clean out unused entries.

```rust
store.prune_stale()?; // safe to run whenever you want a clean delta
```

## Testing tips
- Use temporary directories (`tempfile::tempdir()`) to isolate delta files.
- For migration assertions, inspect `store.schema_version()` and the on-disk JSON to verify both the in-memory value and persisted deltas.
- The crate’s own tests keep per-thread state, so running `cargo test -p settings` in parallel is stable.

---

Internal crate; see the root repository license for terms. Contributions are welcome via workspace pull requests.
