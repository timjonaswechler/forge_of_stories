/*!
Keymap module (Phase 2 – Task 2.3)

This module hosts all keyboard mapping / binding utilities that were previously
located in `services/keymap_binding.rs`.

Goals:
- Provide a single cohesive place for key→action translation.
- Prepare future refactors (e.g. Intent-based reducer, layered contexts, fallback logic).
- Keep the public API stable for existing callers.

Transitional Notes:
- Existing code that did `use crate::services::keymap_binding::...` should be updated to
  `use crate::ui::keymap::{...}`.
- The function names and signatures are unchanged to avoid functional differences
  during this phase.
- A future phase may introduce:
    * Context fallback (popup > page > global)
    * Static label→Intent tables
    * Separation of raw input events vs. semantic intents

Public API (unchanged):
- map_label_to_action(&str) -> Option<Action>
- chord_from_key(KeyEvent) -> Option<String>
- action_from_key(&SettingsStore, &str, KeyEvent) -> Option<Action>
- mappable_entries_for_context(&SettingsStore, &str) -> Vec<(String, Vec<String>)>

Test Coverage:
- Basic smoke tests ensuring mapping + chord formatting.

*/

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use settings::{DeviceFilter, SettingsStore};

use crate::action::{Action, PopupResult};
use crate::theme::Mode;

/// Map a TOML action label (from keymap) to the `Action` enum used by the wizard.
///
/// Extend this mapping as you introduce new action labels in your keymaps.
pub fn map_label_to_action(label: &str) -> Option<Action> {
    match label {
        "Quit" => Some(Action::Quit),
        "Help" => Some(Action::Help),
        "Submit" => Some(Action::Submit),
        "NextField" => Some(Action::Down),
        "PreviousField" => Some(Action::Up),
        "ResetFields" => Some(Action::Refresh),
        "FocusNext" => Some(Action::FocusNext),
        "FocusPrev" | "FocusPrevious" => Some(Action::FocusPrev),
        "Up" => Some(Action::Up),
        "Down" => Some(Action::Down),
        "SwitchInputMode" => Some(Action::SwitchInputMode),
        "Switch" => Some(Action::FocusNext),
        "OpenPopup" => Some(Action::OpenPopup(Box::new(
            crate::components::popups::confirm::ConfirmPopup::new("", ""),
        ))),
        "Cancel" => Some(Action::PopupResult(PopupResult::Cancelled)),
        // Mode control (contextual)
        "ModeCycle" | "ModeNext" => Some(Action::CycleMode),
        "ModeNormal" => Some(Action::SetMode(Mode::Normal)),
        "ModeInsert" => Some(Action::SetMode(Mode::Insert)),
        "ModeVisual" => Some(Action::SetMode(Mode::Visual)),
        // Overlay footer / keymap
        "ToggleKeymap" | "Keymap" => Some(Action::ToggleKeymapOverlay),
        _ => None,
    }
}

/// Convert a crossterm `KeyEvent` into a chord string compatible with the keymap format.
///
/// Examples:
/// - Ctrl+H      => "ctrl+h"
/// - Enter       => "enter"
/// - Shift+Tab   => "shift+tab"
/// - Space       => "space"
/// - Shift+Space => "shift+space"
/// - Char 'A'    => "shift+a"
/// - F1          => "f1"
///
/// Notes:
/// - Characters are normalized to lower-case; SHIFT is represented explicitly in modifiers.
/// - SHIFT for space and regular chars is preserved when pressed.
pub fn chord_from_key(key: KeyEvent) -> Option<String> {
    let (key_str, include_shift) = match key.code {
        KeyCode::Enter => ("enter".to_string(), true),
        KeyCode::Tab => ("tab".to_string(), true),
        KeyCode::Backspace => ("backspace".to_string(), true),
        KeyCode::Esc => ("esc".to_string(), true),
        KeyCode::Up => ("up".to_string(), true),
        KeyCode::Down => ("down".to_string(), true),
        KeyCode::Left => ("left".to_string(), true),
        KeyCode::Right => ("right".to_string(), true),
        KeyCode::Home => ("home".to_string(), true),
        KeyCode::End => ("end".to_string(), true),
        KeyCode::PageUp => ("pageup".to_string(), true),
        KeyCode::PageDown => ("pagedown".to_string(), true),
        KeyCode::F(n) => (format!("f{}", n), true),
        KeyCode::Char(' ') => ("space".to_string(), true),
        KeyCode::Char(ch) => (ch.to_ascii_lowercase().to_string(), true),
        _ => return None,
    };

    let mut mods: Vec<&str> = Vec::new();
    let m = key.modifiers;
    if m.contains(KeyModifiers::CONTROL) {
        mods.push("ctrl");
    }
    if include_shift && m.contains(KeyModifiers::SHIFT) {
        mods.push("shift");
    }
    if m.contains(KeyModifiers::ALT) {
        mods.push("alt");
    }
    if m.contains(KeyModifiers::SUPER) {
        mods.push("meta");
    }

    Some(if mods.is_empty() {
        key_str
    } else {
        format!("{}+{}", mods.join("+"), key_str)
    })
}

/// Resolve the `Action` for a given key event using the keymap export for a `context`.
///
/// Typical usage in the event loop:
/// ```ignore
/// if let Some(action) = action_from_key(&settings_store, context, key_event) {
///     action_tx.send(action).ok();
/// }
/// ```
pub fn action_from_key(store: &SettingsStore, context: &str, key: KeyEvent) -> Option<Action> {
    let chord = chord_from_key(key)?;
    let exported = store.export_keymap_for(DeviceFilter::Keyboard, context);
    let (label, _chords) = exported
        .iter()
        .find(|(_label, chords)| chords.iter().any(|c| c == &chord))?;
    map_label_to_action(label.as_str())
}

/// Convert exported keymap labels+chords for `context` into entries (label, chords)
/// filtered to only those that can map to a concrete `Action`.
pub fn mappable_entries_for_context(
    store: &SettingsStore,
    context: &str,
) -> Vec<(String, Vec<String>)> {
    let exported = store.export_keymap_for(DeviceFilter::Keyboard, context);
    exported
        .into_iter()
        .filter(|(label, _)| map_label_to_action(label).is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    // Simple fake SettingsStore substitute for chord matching tests:
    struct DummyStore;

    impl DummyStore {
        fn store_with(label: &str, chords: &[&str]) -> settings::SettingsStore {
            // We fabricate a SettingsStore by round-tripping JSON through its expected import format.
            // For Phase 2 we avoid deep integration; this is a light smoke test.
            // If constructing a real SettingsStore is non-trivial, these tests can be adapted later.
            let mut store = settings::SettingsStore::default();
            // Provide a minimal keymap injection if the real API allows dynamic updates.
            // If not, these tests can be feature-gated or replaced with pure-unit tests for `chord_from_key`
            // and `map_label_to_action`.
            let device = settings::DeviceFilter::Keyboard;
            for chord in chords {
                store.add_key_binding(device, "testctx", label.to_string(), (*chord).to_string());
            }
            store
        }
    }

    #[test]
    fn chord_basic_ctrl_shift() {
        let k = KeyEvent::new(
            KeyCode::Char('A'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        let chord = chord_from_key(k).unwrap();
        assert_eq!(chord, "ctrl+shift+a");
    }

    #[test]
    fn chord_space_and_shift() {
        let k = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::SHIFT);
        let chord = chord_from_key(k).unwrap();
        assert_eq!(chord, "shift+space");
    }

    #[test]
    fn chord_function_key() {
        let k = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        let chord = chord_from_key(k).unwrap();
        assert_eq!(chord, "f5");
    }

    #[test]
    fn map_label_to_action_known() {
        assert!(matches!(map_label_to_action("Quit"), Some(Action::Quit)));
        assert!(map_label_to_action("NonExisting").is_none());
    }

    #[test]
    fn action_from_key_resolves() {
        let store = DummyStore::store_with("Quit", &["ctrl+q"]);
        let k = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let resolved = action_from_key(&store, "testctx", k);
        assert!(matches!(resolved, Some(Action::Quit)));
    }

    #[test]
    fn mappable_entries_filters_only_mapped() {
        let store = {
            let mut s = settings::SettingsStore::default();
            let device = settings::DeviceFilter::Keyboard;
            s.add_key_binding(device, "ctx", "Quit".into(), "ctrl+q".into());
            s.add_key_binding(device, "ctx", "UnmappedCustom".into(), "ctrl+u".into());
            s
        };
        let entries = mappable_entries_for_context(&store, "ctx");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "Quit");
    }
}
