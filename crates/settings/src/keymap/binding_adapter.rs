/// use leafwing_input_manager::prelude::*;
/// use bevy::prelude::*;
/// use crate::keymap::spec::{BindingSpec, KeystrokeSpec, Modifiers, KeyCodeSpec, ActionSpec};

/// #[derive(Actionlike, Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// pub enum GameAction {
///     Save,
///     OpenMenu,
///     // ...
/// }

/// pub fn apply_bindings_to_input_map(input_map: &mut InputMap<GameAction>, specs: &[BindingSpec]) {
///     for spec in specs {
///         let action = match &spec.action {
///             ActionSpec::NoAction => continue,
///             ActionSpec::Name(name) => match name.as_str() {
///                 "Save" => GameAction::Save,
///                 "OpenMenu" => GameAction::OpenMenu,
///                 _ => continue, // unbekannt → überspringen/loggen
///             },
///             ActionSpec::WithArgs { name, args: _ } => match name.as_str() {
///                 "Save" => GameAction::Save,
///                 _ => continue,
///             },
///         };

///         // Nur Single-keystroke Beispiele (Sequenzen/Chords kannst du später ergänzen)
///         if let Some(first) = spec.keystrokes.first() {
///             if let Some(kb_input) = bevy_input_from(first) {
///                 input_map.insert(kb_input, action);
///             }
///         }
///     }
/// }

/// fn bevy_input_from(ks: &KeystrokeSpec) -> Option<leafwing_input_manager::user_input::UserInput> {
///     use leafwing_input_manager::user_input::{InputKind, UserInput, Modifier};
///     let key = match &ks.key {
///         KeyCodeSpec::Char(c) => InputKind::Keyboard(bevy::input::keyboard::KeyCode::Character(c.to_string())),
///         KeyCodeSpec::Named(name) => {
///             // Mapping-Tabelle anlegen („Escape“, „F1“, …)
///             return None;
///         }
///     };
///     let mut input = UserInput::Single(key);
///     if ks.mods.contains(Modifiers::CTRL) { input = input.with_modifiers([Modifier::Control]); }
///     if ks.mods.contains(Modifiers::ALT) { input = input.with_modifiers([Modifier::Alt]); }
///     if ks.mods.contains(Modifiers::SHIFT) { input = input.with_modifiers([Modifier::Shift]); }
///     if ks.mods.contains(Modifiers::SUPER) { input = input.with_modifiers([Modifier::Super]); }
///     Some(input)
/// }
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BindingSpec {
    pub source: Option<String>,         // z. B. "User", "Default" (optional)
    pub context: Option<String>,        // frei formulierter Kontext, Interpretation in Adapter
    pub keystrokes: Vec<KeystrokeSpec>, // Sequenz ("Ctrl+K Ctrl+S")
    pub action: ActionSpec,             // Name + optionale Args oder NoAction
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeystrokeSpec {
    pub mods: Modifiers,
    pub key: KeyCodeSpec,
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct Modifiers: u8 {
        const CTRL  = 0b0001;
        const ALT   = 0b0010;
        const SHIFT = 0b0100;
        const SUPER = 0b1000; // cmd/win
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyCodeSpec {
    // Mindestens diese Varianten reichen für MVP – erweitern nach Bedarf
    Char(char),    // 'a', 'b', ...
    Named(String), // "Escape", "F1", "ArrowUp", ...
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionSpec {
    NoAction,
    Name(String),
    WithArgs { name: String, args: TomlValue },
}
