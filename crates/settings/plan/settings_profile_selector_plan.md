# Plan: Settings Profile Selector (Game + TUI + TOML)

Derived solely from crates/settings/memories/settings_profile_selector_analysis.md.

## Extracted Model (from memories)
- Modal picker to switch settings profiles with live preview and rollback.
- Fuzzy search with highlighted matches; async background filtering.
- Integrates with SettingsStore to read profiles and a global ActiveSettingsProfileName.
- Immediate preview on navigation; confirm persists, dismiss rolls back.

## Game-Oriented Adaptation
- Profiles represent input/graphics/audio configuration bundles in TOML.
- Sources: built-in defaults + optional per-user custom profiles under `settings/profiles/*.toml`.
- Keep `None` (Disabled) meaning: no active profile (use layered defaults + user overrides only).
- Live preview applies merged settings (Default < BaseProfile < User) without disk write until confirm.

## TUI Integration (ratatui)
- Replace GPUI modal/picker with a ratatui list + search box.
- Implement fuzzy search (e.g., simple subsequence/score) executed on a background task.
- UI shows: profile name, short description (from TOML `profile.description`), and an indicator for active.

## State & Rollback
- State includes: all profile names, original_active, selected_index, selected_name, selection_completed.
- On selection change: compute preview by reloading settings with selected profile layer; do not persist.
- On confirm: write `active_profile = "<name>"` to user settings (toml_edit) and persist.
- On dismiss without confirm: restore `original_active` preview.

## TOML Profile Format
Example profile file:
```toml
[profile]
name = "Performance"
description = "60 FPS target, lower shadows"

[graphics]
shadow_quality = "low"
vsync = true

[input]
base_profile = "WASD"
```
- Minimal required fields: `profile.name`.
- Optional: `profile.description` for UI summary.

## Settings Interaction
- SettingsStore exposes configured profile names and loaders for profile TOML.
- Active profile stored in user settings at `profile.active_profile` (string or none).
- Effective settings = merge(Default, SelectedProfile?, UserOverrides).

## Open Questions
- Persist last previewed profile if user closes by selecting “Confirm” via key or explicit button only?
- Allow per-mode profiles (e.g., server vs client) or single global active profile?
- Should profiles be able to extend another profile (inheritance) to reduce duplication?

## Implementation Steps
1) Define TOML profile schema and discovery under `settings/profiles/`.
2) Extend SettingsStore: list profiles, get active name, compute merged settings for preview.
3) Implement ratatui-based selector with fuzzy search, live preview, confirm/dismiss.
4) Persist `profile.active_profile` on confirm via toml_edit; rollback on cancel.
5) Add tests for preview/rollback and confirm flows using a mock store and temp FS.
