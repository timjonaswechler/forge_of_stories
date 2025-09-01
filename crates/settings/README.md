# Settings and Keymaps

This crate provides layered settings and keymap handling for Forge_of_Stories with:
- Priority-ordered layers (last-wins)
- Deep-merge rules tailored for game settings
- Atomic, durable writes across platforms
- Diff-aware reloads with atomic swaps
- Keymap parsing/merging/export with context semantics
- Platform-aware user config path helpers via the shared `paths` crate

## Quick start

```/dev/null/example.rs#L1-30
use settings::settings::store::{SettingsStore, MergeArraysPolicy, DeviceFilter};
use settings::embedded::{default_settings, default_keymap};

// Build a store with embedded defaults, plus per-user overrides from the platform config dir
let store = SettingsStore::builder("Forge_of_Stories")
    // Built-in defaults (examples):
    .with_embedded_setting_text(default_settings())
    .with_embedded_keymap_text(default_keymap())
    // Optional per-user overrides under the platform config dir:
    .with_user_config_dir()
    // Disable environment layers in noisy environments (CI, etc.):
    .enable_env_layers(false)
    // Optional: change how arrays merge (default is Replace):
    // .merge_arrays_policy(MergeArraysPolicy::Concat)
    .build()?;

// Read effective settings/keymaps
let settings_by_section = store.effective_settings();
let keymaps = store.effective_keymaps();

// Export a keymap view for the current context (global + context; context last-wins)
let kb = store.export_keymap_for(DeviceFilter::Keyboard, "in_game");
```

## Layers and priority

Supported layer kinds (higher index = higher priority):
- Embedded settings text / file assets
- Settings files (optional – missing/empty is neutral)
- Environment prefix layer (optional/disable-able)
- Embedded keymap text / file assets
- Keymap files (optional – missing/empty is neutral)

Rules:
- Ordering matters. Later layers override earlier ones (last-wins) per key/bucket.
- Missing or empty files are treated as neutral.
- Faulty TOML is logged with layer index and path; the layer is treated as neutral.

## Deep-merge rules (Settings)

Top-level sections are merged independently. Within a section:
- Tables: recursively merged (last-wins per key)
- Arrays: replaced entirely by default (last-wins)
- Scalars: replaced (last-wins)

You can opt into different array behavior globally:

```/dev/null/example.rs#L1-7
use settings::settings::store::{SettingsStore, MergeArraysPolicy};

let store = SettingsStore::builder("Forge_of_Stories")
    .merge_arrays_policy(MergeArraysPolicy::Concat) // or Set
    .build()?;
```

Rationale: Replace is the safest default and avoids unbounded growth from stacking layers. Only change this if a section explicitly requires concatenation or set semantics.

## Atomic, durable writes

When writing to disk, writes are made durable and atomic:
- Temp file in the destination directory (same filesystem), created with `create_new`
- Contents are fully written and `sync_all`ed
- POSIX: `rename` then `fsync` on the parent directory to persist the dentry
- Windows: if the target exists, `ReplaceFileW` with `WRITE_THROUGH` for atomic replace; otherwise `rename`

This is crash-resilient and prevents partial files or torn renames.

## Reloading and thread-safety

`reload_all()`:
- Loads and merges without holding locks
- Computes a diff by section and only rebuilds registered snapshots for changed sections
- Atomically swaps `effective_settings` and `effective_keymaps`, then updates only changed snapshots

Registration pattern:

```/dev/null/example.rs#L1-30
use settings::settings::{Settings, SettingsError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
struct NetworkCfg { port: u16 }

struct Network;
impl Settings for Network {
    const SECTION: &'static str = "network";
    type Model = NetworkCfg;
}

store.register::<Network>()?;
let net = store.get::<Network>()?; // Arc<NetworkCfg>
```

## Keymaps

Data model:
- Meta: `devices`, `gamepad`, `mouse_enabled` (simple last-wins)
- Contexts: `global` and additional contexts (e.g. `in_game`, `login`)
- For each context: `action = ["chord", ...]`

Merge/export semantics:
- Last-wins per device bucket (Keyboard/Mouse/Gamepad)
- Export always considers `global` + active context; context last-wins per device
- Stable de-duplication for chords
- Gamepad prefixes: `xbox:`, `dualshock:`, and generic `gp:`/`gamepad:` are supported
  - Filtering by `DeviceFilter::GamepadKind("xbox")` includes both `xbox:` and generic `gp:` chords

Parser normalization:
- Synonyms: `esc|escape`, `enter|return`, `space|spc`
- `?` remains a literal character; there is no layout expansion (e.g., QWERTY `shift+/` vs QWERTZ `shift+ß`).
  This should be resolved in the input layer against the OS keyboard layout.

Example keymap TOML:

```/dev/null/keymap.toml#L1-14
[meta]
devices = ["keyboard", "gamepad"]

[global]
open_menu = ["esc", "gp:start"]

[in_game]
open_menu = ["esc"]            # overrides only keyboard bucket
jump      = ["space", "gp:a"]
action    = ["?", "gp:x"]      # literal question mark
```

Exporting:

```/dev/null/example.rs#L1-15
use settings::settings::store::DeviceFilter;

// Keyboard view (global + in_game, context last-wins per device)
let kb = store.export_keymap_for(DeviceFilter::Keyboard, "in_game");
// Example: { "open_menu": ["esc"], "jump": ["space"], "action": ["?"] }

// Xbox view (includes generic gp: and xbox:)
let gp = store.export_keymap_for(DeviceFilter::GamepadKind("xbox".into()), "in_game");
// Example: { "open_menu": ["gp:start"], "jump": ["gp:a"], "action": ["gp:x"] }
```

## Paths and user config

Use the shared `paths` crate for platform-correct directories:
- Linux/BSD: `~/.config/Forge_of_Stories`, local data under XDG paths
- macOS: `~/Library/Application Support/Forge_of_Stories` (data), `~/Library/Logs/Forge_of_Stories` (logs)
- Windows: `%APPDATA%`/`%LOCALAPPDATA%` as resolved by `dirs`

Builder helper:
- `.with_user_config_dir()` adds two optional layers:
  - `<config_dir>/<app_id>/settings.toml`
  - `<config_dir>/<app_id>/keymap.toml`

```/dev/null/example.rs#L1-10
let store = SettingsStore::builder("Forge_of_Stories")
    .with_user_config_dir()
    .build()?;
```

## Environment layers

Environment layers (e.g., `APP__NETWORK__PORT` → `[network].port`) are optional and can be disabled:

```/dev/null/example.rs#L1-10
let store = SettingsStore::builder("Forge_of_Stories")
    .with_user_config_dir()
    .enable_env_layers(false)
    .build()?;
```

When disabled, env layers are ignored and a warning is logged.

## Error handling and logging

Faulty or unreadable TOML layers:
- The error is logged with layer index and path (where applicable)
- The layer is treated as neutral (empty table), so the application continues

Logging is expected to be provided by the workspace (e.g., `log`/`tracing`).

## Notes and future improvements

- Consider introducing explicit `char:` vs. `key:` prefixes for keymaps to distinguish layout-agnostic character bindings from physical key bindings.
- Section-specific array merge policies can be added later if a specific section requires `Concat` or `Set`. The default remains `Replace`.