# Plan: Settings Profile Selector (RS) → Game TUI + TOML

Derived solely from crates/settings/memories/settings_profile_selector_rs_analysis.md.

## Extracted Model (from memories)
- GPUI-based modal with a Picker delegate, fuzzy search, and live profile preview.
- Global ActiveSettingsProfileName updated on navigation; confirm persists, dismiss rolls back.
- Profiles sourced from SettingsStore; includes a Disabled (None) option.

## Game/TOML Adaptation
- Keep UX: live preview while navigating, rollback on cancel, explicit confirm to persist.
- Replace GPUI/Picker with ratatui components (search input + list with highlight positions).
- Profiles are TOML files under `settings/profiles/*.toml`; `Disabled` means no active profile.
- Active profile is stored at `profile.active_profile` in user TOML.

## Data & State
- State: `matches: Vec<Match>`, `profile_names: Vec<Option<String>>`, `original_active`, `selected_index`, `selected_name`, `selection_completed`.
- Match contains: candidate_id, score, positions, display string.
- Display: use profile name or "Disabled" for None.

## Fuzzy Search
- Background task computes matches over names; empty query returns all with score 0.
- Update selected_index clamped to bounds; update preview after every recompute.
- Provide highlight positions to render emphasized substrings in the TUI.

## Live Preview & Persistence
- On navigation: recalculate effective settings with selected profile layered (Default < Profile < User overrides); do not write disk.
- On confirm: write `profile.active_profile = "<name>"` via a toml_edit updater; set `selection_completed = true`.
- On dismiss without confirm: restore `original_active` as the preview active profile.

## Profile TOML Schema
```toml
[profile]
name = "<string>"
description = "<optional>"
# Other sections such as [graphics], [input], etc., overlay onto settings
```

## Open Questions
- Single global active profile or per-mode (client/server) profiles?
- Should profiles support inheritance (extend = "BaseProfile")?
- Persist last search query between openings?

## Implementation Steps
1) Define profile discovery (list names from `settings/profiles/*.toml`).
2) Implement TUI selector: search box, list, highlight; background fuzzy search.
3) Wire live preview by reloading layered settings using the selected profile.
4) Persist active profile on confirm via toml_edit; rollback on cancel.
5) Add integration tests covering preview, confirm, cancel, navigation.
