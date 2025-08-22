# Plan: Base Keymap Settings (Game + TOML)

Derived solely from crates/settings/memories/base_keymap_setting_rs_analysis.md.

## Extracted Model (from memories)
- Enum-based preset selector with a default and Display labels.
- Platform-aware OPTIONS list and asset_path mapping.
- Optional file content, layered load order: user > server > default.
- JSON-centric assets (per-OS .json) and schemars integration.

## Game-Oriented Adaptation
- Rename concept: BaseKeymap → BaseInputProfile (domain: controls, not editors).
- Presets: WASD, ArrowKeys, GamepadXbox, GamepadDualShock, GamepadGeneric, None.
- Default: WASD on desktop; allow per-OS label tweaks (Option+Command vs Ctrl).
- Platform differences: Keep OPTIONS per-OS; add device-aware variants (e.g., DualShock on macOS).
- Asset paths: switch to .toml assets, e.g. keymaps/{os}/wasd.toml, keymaps/common/gamepad_xbox.toml.
- Keep names()/from_name() for UI lists; prefer singular from_name.

## TOML Structure
Example user config (layered key path suggestion):
```toml
[input]
base_profile = "WASD"       # or "GamepadXbox", "None"
mouse_sensitivity = 1.0
invert_y = false
```

Example keymap asset (embedded or loaded):
```toml
# keymaps/common/wasd.toml
[[bindings]]
action = "move_forward"
keys = ["W"]

[[bindings]]
action = "pause"
keys = ["Escape"]

[[bindings]]
action = "jump"
keys = ["Space"]
```
Gamepad example:
```toml
# keymaps/common/gamepad_xbox.toml
[[bindings]]
action = "jump"
gamepad = { button = "South" }
```

## Loading & Priority
- FileContent remains Option<Self>; overall layer order unchanged (user > server > default).
- asset_path() returns .toml paths; VSCode/Editor-specific entries removed.
- For None, load no preset; only user overrides apply.

## Migration Notes (JSON → TOML)
- Drop schemars/JsonSchema for this setting; use serde + toml for I/O.
- If needed, accept legacy JSON once during import and rewrite to TOML.
- Provide sample TOML under keymaps/{os}/ to replace prior JSON.

## Open Questions (for you)
- Confirm preset list: WASD, ArrowKeys, GamepadXbox, GamepadDualShock, GamepadGeneric, None.
- Do we need per-layout keyboard variants (QWERTZ/AZERTY)? If yes, add layout suffixes.
- Should mouse settings live under [input] or [input.mouse]?
- Embed assets at compile time or load from disk for modding?

## Implementation Steps
1) Define BaseInputProfile enum + Display labels.
2) Implement OPTIONS/asset_path() with .toml targets per OS.
3) Wire setting key to input.base_profile; keep Option<Self>.
4) Parse TOML assets into in-memory bindings; merge with user overrides.
5) Add migration helper to import legacy JSON if encountered.
