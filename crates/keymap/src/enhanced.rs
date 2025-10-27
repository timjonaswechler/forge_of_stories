//! Helpers for integrating keymap data with `bevy_enhanced_input`.
//!
//! The functions in this module operate on descriptor data and return
//! `bevy_enhanced_input` structures so that downstream crates can spawn
//! actions, contexts, and bindings using their own application logic.

use crate::binding::KeyBinding;
use crate::keystroke::{Keystroke, Modifiers};
use crate::spec::{BindingDescriptor, BindingInputDescriptor, modifiers_from_strings};
use bevy::input::{
    gamepad::GamepadAxis, gamepad::GamepadButton, keyboard::KeyCode, mouse::MouseButton,
};
use bevy_enhanced_input::binding::{Binding, mod_keys::ModKeys};
use std::error::Error;
use std::fmt;

/// Error returned when converting descriptors into enhanced input bindings.
#[derive(Debug)]
pub enum ConversionError {
    /// The binding contains more than one keystroke (multi-step sequence).
    MultiStepSequence(usize),
    /// The keystroke's key could not be mapped to a [`KeyCode`].
    UnsupportedKey(String),
    /// The mouse button string could not be mapped.
    UnsupportedMouseButton(String),
    /// The gamepad button string could not be mapped.
    UnsupportedGamepadButton(String),
    /// The gamepad axis string could not be mapped.
    UnsupportedGamepadAxis(String),
    /// Invalid modifier string.
    InvalidModifier(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MultiStepSequence(len) => {
                write!(f, "multi-step sequences are not supported (len = {len})")
            }
            Self::UnsupportedKey(key) => write!(f, "unsupported key {key:?}"),
            Self::UnsupportedMouseButton(button) => {
                write!(f, "unsupported mouse button {button:?}")
            }
            Self::UnsupportedGamepadButton(button) => {
                write!(f, "unsupported gamepad button {button:?}")
            }
            Self::UnsupportedGamepadAxis(axis) => {
                write!(f, "unsupported gamepad axis {axis:?}")
            }
            Self::InvalidModifier(name) => write!(f, "invalid modifier name {name:?}"),
        }
    }
}

impl Error for ConversionError {}

/// Convert a [`BindingDescriptor`] into an enhanced-input [`Binding`].
///
/// Returns `Ok(None)` when the descriptor represents an explicit `None` binding
/// or has no input attached.
pub fn binding_descriptor_to_binding(
    descriptor: &BindingDescriptor,
) -> Result<Option<Binding>, ConversionError> {
    let input = match descriptor.input.as_ref() {
        Some(input) => input,
        None => return Ok(None),
    };

    binding_input_to_binding(input)
}

/// Convert a binding input descriptor into an enhanced-input [`Binding`].
pub fn binding_input_to_binding(
    input: &BindingInputDescriptor,
) -> Result<Option<Binding>, ConversionError> {
    match input {
        BindingInputDescriptor::Keyboard { sequence }
        | BindingInputDescriptor::KeyboardChord { sequence } => {
            keyboard_sequence_to_binding(sequence).map(Some)
        }
        BindingInputDescriptor::MouseButton { button, modifiers } => {
            let button = mouse_button_from_str(button)
                .ok_or_else(|| ConversionError::UnsupportedMouseButton(button.clone()))?;
            let modifiers = modifiers_from_strings(modifiers)
                .map_err(|err| ConversionError::InvalidModifier(err.to_string()))?;
            let mod_keys = mod_keys_from_modifiers(&modifiers);
            Ok(Some(Binding::MouseButton { button, mod_keys }))
        }
        BindingInputDescriptor::MouseMotion { modifiers } => {
            let modifiers = modifiers_from_strings(modifiers)
                .map_err(|err| ConversionError::InvalidModifier(err.to_string()))?;
            let mod_keys = mod_keys_from_modifiers(&modifiers);
            Ok(Some(Binding::MouseMotion { mod_keys }))
        }
        BindingInputDescriptor::MouseWheel { modifiers } => {
            let modifiers = modifiers_from_strings(modifiers)
                .map_err(|err| ConversionError::InvalidModifier(err.to_string()))?;
            let mod_keys = mod_keys_from_modifiers(&modifiers);
            Ok(Some(Binding::MouseWheel { mod_keys }))
        }
        BindingInputDescriptor::GamepadButton { button, .. } => {
            let button = gamepad_button_from_str(button)
                .ok_or_else(|| ConversionError::UnsupportedGamepadButton(button.clone()))?;
            Ok(Some(Binding::GamepadButton(button)))
        }
        BindingInputDescriptor::GamepadAxis { axis, .. } => {
            let axis = gamepad_axis_from_str(axis)
                .ok_or_else(|| ConversionError::UnsupportedGamepadAxis(axis.clone()))?;
            Ok(Some(Binding::GamepadAxis(axis)))
        }
        BindingInputDescriptor::AnyKey { .. } => Ok(Some(Binding::AnyKey)),
        BindingInputDescriptor::None => Ok(Some(Binding::None)),
    }
}

/// Convert a [`KeyBinding`] that represents a single keyboard press into a
/// `bevy_enhanced_input` [`Binding`].
pub fn binding_to_keyboard(binding: &KeyBinding) -> Result<Binding, ConversionError> {
    match binding.keystrokes().len() {
        0 => Err(ConversionError::UnsupportedKey(String::from(
            "empty keystroke sequence",
        ))),
        1 => keystroke_to_keyboard(&binding.keystrokes()[0]),
        len => Err(ConversionError::MultiStepSequence(len)),
    }
}

/// Convert an individual [`Keystroke`] to a [`Binding::Keyboard`] variant.
pub fn keystroke_to_keyboard(keystroke: &Keystroke) -> Result<Binding, ConversionError> {
    keyboard_sequence_to_binding(std::slice::from_ref(keystroke))
}

fn keyboard_sequence_to_binding(sequence: &[Keystroke]) -> Result<Binding, ConversionError> {
    if sequence.is_empty() {
        return Err(ConversionError::UnsupportedKey("empty sequence".into()));
    }
    if sequence.len() != 1 {
        return Err(ConversionError::MultiStepSequence(sequence.len()));
    }

    let keystroke = &sequence[0];
    let key = keycode_from_str(&keystroke.key)
        .ok_or_else(|| ConversionError::UnsupportedKey(keystroke.key.clone()))?;
    let mod_keys = mod_keys_from_modifiers(&keystroke.modifiers);

    Ok(Binding::Keyboard { key, mod_keys })
}

fn mod_keys_from_modifiers(modifiers: &Modifiers) -> ModKeys {
    let mut mod_keys = ModKeys::empty();
    if modifiers.ctrl {
        mod_keys |= ModKeys::CONTROL;
    }
    if modifiers.shift {
        mod_keys |= ModKeys::SHIFT;
    }
    if modifiers.alt {
        mod_keys |= ModKeys::ALT;
    }
    if modifiers.cmd {
        mod_keys |= ModKeys::SUPER;
    }
    mod_keys
}

fn keycode_from_str(key: &str) -> Option<KeyCode> {
    Some(match key {
        // Letters
        "a" => KeyCode::KeyA,
        "b" => KeyCode::KeyB,
        "c" => KeyCode::KeyC,
        "d" => KeyCode::KeyD,
        "e" => KeyCode::KeyE,
        "f" => KeyCode::KeyF,
        "g" => KeyCode::KeyG,
        "h" => KeyCode::KeyH,
        "i" => KeyCode::KeyI,
        "j" => KeyCode::KeyJ,
        "k" => KeyCode::KeyK,
        "l" => KeyCode::KeyL,
        "m" => KeyCode::KeyM,
        "n" => KeyCode::KeyN,
        "o" => KeyCode::KeyO,
        "p" => KeyCode::KeyP,
        "q" => KeyCode::KeyQ,
        "r" => KeyCode::KeyR,
        "s" => KeyCode::KeyS,
        "t" => KeyCode::KeyT,
        "u" => KeyCode::KeyU,
        "v" => KeyCode::KeyV,
        "w" => KeyCode::KeyW,
        "x" => KeyCode::KeyX,
        "y" => KeyCode::KeyY,
        "z" => KeyCode::KeyZ,

        // Digits
        "0" => KeyCode::Digit0,
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "4" => KeyCode::Digit4,
        "5" => KeyCode::Digit5,
        "6" => KeyCode::Digit6,
        "7" => KeyCode::Digit7,
        "8" => KeyCode::Digit8,
        "9" => KeyCode::Digit9,

        // Function keys
        "f1" => KeyCode::F1,
        "f2" => KeyCode::F2,
        "f3" => KeyCode::F3,
        "f4" => KeyCode::F4,
        "f5" => KeyCode::F5,
        "f6" => KeyCode::F6,
        "f7" => KeyCode::F7,
        "f8" => KeyCode::F8,
        "f9" => KeyCode::F9,
        "f10" => KeyCode::F10,
        "f11" => KeyCode::F11,
        "f12" => KeyCode::F12,
        "f13" => KeyCode::F13,
        "f14" => KeyCode::F14,
        "f15" => KeyCode::F15,
        "f16" => KeyCode::F16,
        "f17" => KeyCode::F17,
        "f18" => KeyCode::F18,
        "f19" => KeyCode::F19,
        "f20" => KeyCode::F20,
        "f21" => KeyCode::F21,
        "f22" => KeyCode::F22,
        "f23" => KeyCode::F23,
        "f24" => KeyCode::F24,

        // Special keys
        "escape" => KeyCode::Escape,
        "space" => KeyCode::Space,
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,

        // Arrow keys
        "up" | "arrowup" => KeyCode::ArrowUp,
        "down" | "arrowdown" => KeyCode::ArrowDown,
        "left" | "arrowleft" => KeyCode::ArrowLeft,
        "right" | "arrowright" => KeyCode::ArrowRight,

        // Punctuation
        "-" => KeyCode::Minus,
        "=" => KeyCode::Equal,
        "[" => KeyCode::BracketLeft,
        "]" => KeyCode::BracketRight,
        "\\" => KeyCode::Backslash,
        ";" => KeyCode::Semicolon,
        "'" => KeyCode::Quote,
        "," => KeyCode::Comma,
        "." => KeyCode::Period,
        "/" => KeyCode::Slash,
        "`" => KeyCode::Backquote,

        // Numpad
        "numpad0" => KeyCode::Numpad0,
        "numpad1" => KeyCode::Numpad1,
        "numpad2" => KeyCode::Numpad2,
        "numpad3" => KeyCode::Numpad3,
        "numpad4" => KeyCode::Numpad4,
        "numpad5" => KeyCode::Numpad5,
        "numpad6" => KeyCode::Numpad6,
        "numpad7" => KeyCode::Numpad7,
        "numpad8" => KeyCode::Numpad8,
        "numpad9" => KeyCode::Numpad9,
        "numpad+" => KeyCode::NumpadAdd,
        "numpad-" => KeyCode::NumpadSubtract,
        "numpad*" => KeyCode::NumpadMultiply,
        "numpad/" => KeyCode::NumpadDivide,
        "numpad." => KeyCode::NumpadDecimal,
        "numpadenter" => KeyCode::NumpadEnter,
        "numpad=" => KeyCode::NumpadEqual,

        // Other keys
        "capslock" => KeyCode::CapsLock,
        "numlock" => KeyCode::NumLock,
        "scrolllock" => KeyCode::ScrollLock,
        "printscreen" => KeyCode::PrintScreen,
        "pause" => KeyCode::Pause,

        _ => return None,
    })
}

fn mouse_button_from_str(button: &str) -> Option<MouseButton> {
    match button.to_lowercase().as_str() {
        "left" => Some(MouseButton::Left),
        "right" => Some(MouseButton::Right),
        "middle" => Some(MouseButton::Middle),
        other if other.starts_with("other") => {
            let value = other.split(':').nth(1)?.parse().ok()?;
            Some(MouseButton::Other(value))
        }
        _ => None,
    }
}

fn gamepad_button_from_str(button: &str) -> Option<GamepadButton> {
    match button.to_lowercase().as_str() {
        "south" => Some(GamepadButton::South),
        "east" => Some(GamepadButton::East),
        "west" => Some(GamepadButton::West),
        "north" => Some(GamepadButton::North),
        "c" => Some(GamepadButton::C),
        "z" => Some(GamepadButton::Z),
        "leftshoulder" | "l1" => Some(GamepadButton::LeftTrigger),
        "leftshoulder2" | "l1_2" => Some(GamepadButton::LeftTrigger2),
        "rightshoulder" | "r1" => Some(GamepadButton::RightTrigger),
        "rightshoulder2" | "r1_2" => Some(GamepadButton::RightTrigger2),
        "lefttrigger" | "l2" => Some(GamepadButton::LeftTrigger2),
        "righttrigger" | "r2" => Some(GamepadButton::RightTrigger2),
        "select" | "back" => Some(GamepadButton::Select),
        "start" => Some(GamepadButton::Start),
        "mode" => Some(GamepadButton::Mode),
        "leftstick" | "leftthumb" => Some(GamepadButton::LeftThumb),
        "rightstick" | "rightthumb" => Some(GamepadButton::RightThumb),
        "dpadup" => Some(GamepadButton::DPadUp),
        "dpaddown" => Some(GamepadButton::DPadDown),
        "dpadleft" => Some(GamepadButton::DPadLeft),
        "dpadright" => Some(GamepadButton::DPadRight),
        other if other.starts_with("other") => {
            let value = other.split(':').nth(1)?.parse().ok()?;
            Some(GamepadButton::Other(value))
        }
        _ => None,
    }
}

fn gamepad_axis_from_str(axis: &str) -> Option<GamepadAxis> {
    match axis.to_lowercase().as_str() {
        "leftstickx" => Some(GamepadAxis::LeftStickX),
        "leftsticky" => Some(GamepadAxis::LeftStickY),
        "leftz" | "lefttrigger" | "l2" => Some(GamepadAxis::LeftZ),
        "rightstickx" => Some(GamepadAxis::RightStickX),
        "rightsticky" => Some(GamepadAxis::RightStickY),
        "rightz" | "righttrigger" | "r2" => Some(GamepadAxis::RightZ),
        other if other.starts_with("other") => {
            let value = other.split(':').nth(1)?.parse().ok()?;
            Some(GamepadAxis::Other(value))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::ActionId;
    use crate::spec::BindingDescriptor;

    fn keyboard_descriptor(seq: &str) -> BindingDescriptor {
        BindingDescriptor {
            action_id: Some(ActionId::from("test")),
            meta: None,
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                crate::parse_keystroke_sequence(seq).unwrap(),
            )),
        }
    }

    #[test]
    fn convert_keyboard_binding() {
        let binding = keyboard_descriptor("ctrl-s");
        let result = binding_descriptor_to_binding(&binding).unwrap().unwrap();

        match result {
            Binding::Keyboard { key, mod_keys } => {
                assert_eq!(key, KeyCode::KeyS);
                assert!(mod_keys.contains(ModKeys::CONTROL));
            }
            other => panic!("unexpected binding {other:?}"),
        }
    }

    #[test]
    fn reject_keyboard_chord() {
        let binding = keyboard_descriptor("ctrl-k ctrl-t");
        let err = binding_descriptor_to_binding(&binding).unwrap_err();
        matches!(err, ConversionError::MultiStepSequence(2));
    }

    #[test]
    fn convert_mouse_button() {
        let descriptor = BindingInputDescriptor::MouseButton {
            button: "left".into(),
            modifiers: vec!["shift".into()],
        };
        let binding = binding_input_to_binding(&descriptor).unwrap().unwrap();
        match binding {
            Binding::MouseButton { button, mod_keys } => {
                assert_eq!(button, MouseButton::Left);
                assert!(mod_keys.contains(ModKeys::SHIFT));
            }
            other => panic!("unexpected binding {other:?}"),
        }
    }
}
