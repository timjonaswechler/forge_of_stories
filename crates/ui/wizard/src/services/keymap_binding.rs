use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use settings::{DeviceFilter, SettingsStore};

use crate::action::{Action, PopupResult};

/// Map a TOML action label (from keymap) to the `Action` enum used by the wizard.
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
        "FocusPrev" => Some(Action::FocusPrev),
        "FocusPrevious" => Some(Action::FocusPrev),
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
        "ModeNormal" => Some(Action::SetMode(crate::theme::Mode::Normal)),
        "ModeInsert" => Some(Action::SetMode(crate::theme::Mode::Insert)),
        "ModeVisual" => Some(Action::SetMode(crate::theme::Mode::Visual)),
        // Overlay footer / keymap
        "ToggleKeymap" | "Keymap" => Some(Action::ToggleKeymapOverlay),
        _ => None,
    }
}

/// Convert a crossterm `KeyEvent` into a chord string compatible with our keymap format.
///
/// Examples:
/// - Ctrl+H => "ctrl+h"
/// - Enter  => "enter"
/// - Shift+Tab => "shift+tab"
/// - Space / Shift+Space => "space" / "shift+space"
/// - Char 'a' / Shift+'A' => "a" / "shift+a"
/// - F1 => "f1"
///
/// Notes:
/// - Zeichen (KeyCode::Char) werden zu lower-case normalisiert; SHIFT wird in den Modifiers geführt.
/// - Für Space und Zeichen wird SHIFT bewusst berücksichtigt (wie vom Nutzer gewünscht).
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

/// Resolve the `Action` for a given key event using the current keymap export for `context`.
/// Returns `None` if the chord does not match any configured binding for the keyboard device.
///
/// Typical usage in your event loop:
/// - let context = page.keymap_context();
/// - if let Some(action) = action_from_key(&self.settings, context, key) { action_tx.send(action).ok(); }
pub fn action_from_key(store: &SettingsStore, context: &str, key: KeyEvent) -> Option<Action> {
    let chord = chord_from_key(key)?;
    let exported = store.export_keymap_for(DeviceFilter::Keyboard, context);
    let (label, _chords) = exported
        .iter()
        .find(|(_label, chords)| chords.iter().any(|c| c == &chord))?;
    map_label_to_action(label.as_str())
}

/// Convert exported keymap labels+chords for `context` into concrete `Action`s.
/// Only entries that can be mapped are returned.
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
