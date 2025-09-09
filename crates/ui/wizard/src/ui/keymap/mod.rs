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

use crate::action::{Action, UiOutcome};
use crate::theme::Mode;
pub mod mapper;
use self::mapper::intent_for_label;

/// Map a TOML action label (from keymap) to the `Action` enum used by the wizard.
///
/// Extend this mapping as you introduce new action labels in your keymaps.
pub fn map_label_to_action(label: &str) -> Option<Action> {
    // Phase 7.1: legacy function now delegates to static intent table in mapper.rs
    intent_for_label(label)
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

    #[test]
    fn chord_basic_ctrl_shift() {
        let k = KeyEvent::new(
            KeyCode::Char('A'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        let chord = super::chord_from_key(k).unwrap();
        assert_eq!(chord, "ctrl+shift+a");
    }

    #[test]
    fn chord_space_and_shift() {
        let k = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::SHIFT);
        let chord = super::chord_from_key(k).unwrap();
        assert_eq!(chord, "shift+space");
    }

    #[test]
    fn chord_function_key() {
        let k = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
        let chord = super::chord_from_key(k).unwrap();
        assert_eq!(chord, "f5");
    }

    #[test]
    fn map_label_to_action_known() {
        assert!(matches!(
            super::map_label_to_action("Quit"),
            Some(super::Action::Quit)
        ));
        assert!(super::map_label_to_action("NonExisting").is_none());
    }
}
