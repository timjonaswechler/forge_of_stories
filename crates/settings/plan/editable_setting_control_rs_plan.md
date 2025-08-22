# Plan: EditableSettingControl for Game (TOML + TUI)

Derived solely from crates/settings/memories/editable_setting_control_rs_analysis.md.

## Extracted Model (from memories)
- Trait EditableSettingControl with associated Value and Settings types.
- UI coupling via RenderOnce and gpui::App context; SharedString labels.
- Default write() that updates a settings file through an abstract Fs and update_settings_file.
- Layered settings model implied by Settings trait and FileContent mutation.

## Goals for Our Game
- Use TOML as the on-disk format (preserve comments where possible).
- Decouple from gpui; integrate with our TUI (ratatui/crossterm) and server CLI.
- Keep layered sources (user > server > default) and type safety.
- Make IO testable (mockable FS) and safe (atomic writes).

## Proposed Trait Shape (agnostic to UI framework)
- Keep name(), read(), apply(), write() but remove gpui::App and RenderOnce coupling.
- Replace SharedString with String (or Cow<'static, str> if needed).
- Accept a SettingsStore handle (our abstraction) instead of gpui::App in read/apply/write.
- Value: Send + 'static; Settings: our Settings trait with FileContent.
- Default write(): calls a TOML-aware update function (atomic, comment-preserving).

Pseudo-signature sketch:
```rust
pub trait EditableSettingControl {
    type Value: Send + 'static;
    type Settings: Settings; // has associated FileContent

    fn name(&self) -> String;
    fn read(store: &SettingsStore) -> Self::Value;
    fn apply(content: &mut <Self::Settings as Settings>::FileContent, value: Self::Value);
    fn write(value: Self::Value, store: &SettingsStore) {
        update_settings_toml::<Self::Settings>(store, move |content| {
            Self::apply(content, value);
        });
    }
}
```

## TOML Update Pipeline
- update_settings_toml<T>: loads current TOML, parses (toml_edit), applies closure, writes back atomically.
- Respect layered resolution: modify the highest-priority writable layer (user over server/default).
- Provide rollback on error and minimal diffs to reduce churn.

## UI Integration (ratatui)
- Keep trait UI-agnostic; create adapters:
  - TuiControl<T: EditableSettingControl>: bridges events -> Value -> write().
  - Render responsibilities live outside the trait to avoid framework lock-in.

## Testing Strategy
- Mock SettingsStore and filesystem.
- Unit-test apply() per control; integration-test write() happy/edge paths.
- Fuzz minimal TOML edits around arrays/tables to ensure stability.

## Migration Notes
- No gpui::App; no RenderOnce; no SharedString.
- Replaces update_settings_file with update_settings_toml.
- Ensure Send bounds preserved for background tasks (tokio spawn_blocking if needed).

## Open Questions
- Do we want write() to be async for the wizard (tokio)?
- Should name() be localized now or later (string keys instead of literals)?
- Confirm layered write target: always user layer unless a control is admin-only.

## Next Steps
1) Define SettingsStore and update_settings_toml API surface.
2) Port one concrete control as a reference (e.g., input.base_profile selector).
3) Add a small ratatui adapter that calls T::read() and T::write().
4) Document control authoring guidelines (naming, error handling, tests).
