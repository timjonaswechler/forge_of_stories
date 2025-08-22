# Plan: settings.rs Orchestrator → Game/TOML

Derived solely from crates/settings/memories/settings_rs_analysis.md.

## Extracted Model (from memories)
- Central orchestrator module re-exporting submodules (keymap, file watch, JSON editor, store, etc.).
- Uses gpui `App` globals; embeds assets via `rust_embed` (settings + keymaps JSON).
- Initializes SettingsStore, loads default JSON, registers BaseKeymap, observes active profile name.
- Provides platform-specific default keymap paths.

## Game/TOML Adaptation
- Replace JSON with TOML for defaults and keymaps.
- Decouple from `gpui::App` globals; expose an app-agnostic initializer usable by server and TUI.
- Keep asset embedding optional: support both embedded defaults and disk overrides for modding.
- Replace `BaseKeymap` registration with game-centric `BaseInputProfile` (see keymap plans).
- Replace gpui-based profile observation with an internal notifier (channel/broadcast) in SettingsStore.

## Asset Layout (TOML)
- Embedded defaults (optional): assets/settings/default.toml
- Platform keymaps: assets/keymaps/default-macos.toml, assets/keymaps/default-linux.toml
- Vim keymap removed (editor-specific); optional game presets under assets/keymaps/common/*.toml

## Initialization Flow
```rust
pub struct InitOptions {
  pub use_embedded_assets: bool,
  pub platform: Platform,          // MacOS | Linux | Windows
}

pub fn init(store: &mut SettingsStore, opts: InitOptions) -> Result<()> {
  // 1) Load default settings TOML (embedded or disk) and set as defaults
  // 2) Register BaseInputProfile presets (names, asset paths)
  // 3) Set active profile from persisted user settings if present
  // 4) Start profile observation (store-internal notifier)
}
```
- No gpui `Global`; caller holds `SettingsStore` or shares via Arc.

## Platform Defaults
- Consts for platform-specific keymap paths now point to .toml files and include Windows explicitly.
- Conditional compilation for default constants remains acceptable.

## Public Re-Exports
- Re-export only TOML-based components (settings_file, store, keymap, editable controls, base input profile).
- Remove or gate JSON-specific exports behind a legacy feature if needed.

## Open Questions
- Embed assets by default, or prefer disk to allow modding without rebuilds?
- Where to persist `active_profile` (key path)? Suggested: `profile.active_profile` in user TOML.
- Provide Windows-specific default keymap file separate from Linux?

## Implementation Steps
1) Create TOML assets and update constants for platform defaults.
2) Implement `InitOptions` and `init()` that does not depend on gpui.
3) Swap BaseKeymap registration for BaseInputProfile registration.
4) Add SettingsStore notifier for active profile changes and remove gpui observer.
5) Update re-exports to reflect TOML-centric modules and types.
