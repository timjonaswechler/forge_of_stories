//! Serializable descriptors for defining actions, contexts, and bindings.
//!
//! These data structures capture the full intent of an input layout so that
//! defaults can be authored in Rust while user overrides live in JSON. After
//! merging, the descriptors can be translated into `bevy_enhanced_input`
//! entities.

use crate::binding::{ActionId, KeyBinding, KeyBindingMetaIndex};
use crate::keystroke::{Keystroke, Modifiers};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Complete specification of actions, contexts, and bindings.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeymapSpec {
    /// Actions that can be referenced by bindings.
    #[serde(default)]
    pub actions: Vec<ActionDescriptor>,
    /// Binding descriptions connecting inputs to actions.
    #[serde(default)]
    pub bindings: Vec<BindingDescriptor>,
}

/// Description of an input action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionDescriptor {
    /// Identifier of the action.
    pub id: ActionId,
    /// Optional output type hint used when generating `InputAction`.
    #[serde(default)]
    #[cfg_attr(
        not(feature = "binary"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub output: Option<String>,
    /// Optional action-level modifiers to register.
    #[serde(default)]
    #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
    pub modifiers: Vec<RegisteredComponent>,
    /// Optional action-level conditions to register.
    #[serde(default)]
    #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
    pub conditions: Vec<RegisteredComponent>,
    /// Additional settings serialized as arbitrary JSON payload.
    #[serde(default)]
    #[cfg_attr(
        not(feature = "binary"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub settings: Option<Value>,
}

/// Describes how a binding should be spawned.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BindingDescriptor {
    /// Target action. `None` disables the keystroke sequence.
    #[serde(default)]
    #[cfg_attr(
        not(feature = "binary"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub action_id: Option<ActionId>,
    /// Source metadata controlling precedence (defaults to `DEFAULT`).
    #[serde(default)]
    #[cfg_attr(
        not(feature = "binary"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub meta: Option<KeyBindingMetaIndex>,
    /// Optional binding-specific modifiers.
    #[serde(default)]
    #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
    pub modifiers: Vec<RegisteredComponent>,
    /// Optional binding-specific conditions.
    #[serde(default)]
    #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
    pub conditions: Vec<RegisteredComponent>,
    /// Additional binding settings.
    #[serde(default)]
    #[cfg_attr(
        not(feature = "binary"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub settings: Option<Value>,
    /// The physical input used to trigger the binding.
    #[serde(default)]
    #[cfg_attr(
        not(feature = "binary"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub input: Option<BindingInputDescriptor>,
}

impl BindingDescriptor {
    /// Returns the metadata or [`KeyBindingMetaIndex::DEFAULT`] if absent.
    pub fn meta(&self) -> KeyBindingMetaIndex {
        self.meta.unwrap_or(KeyBindingMetaIndex::DEFAULT)
    }

    /// Extract the keyboard keystroke sequence if the input is keyboard-based.
    pub fn keyboard_sequence(&self) -> Option<&[Keystroke]> {
        match self.input.as_ref()? {
            BindingInputDescriptor::Keyboard { sequence } => Some(sequence.as_slice()),
            BindingInputDescriptor::KeyboardChord { sequence } => Some(sequence.as_slice()),
            _ => None,
        }
    }

    /// Convert the descriptor into a [`KeyBinding`] for the legacy keymap engine,
    /// if it targets the keyboard.
    pub fn to_key_binding(&self) -> Result<Option<KeyBinding>> {
        let sequence = match self.keyboard_sequence() {
            Some(seq) => seq,
            None => return Ok(None),
        };

        let action_id = self.action_id.clone();

        Ok(Some(
            KeyBinding::new(sequence.to_vec(), action_id).with_meta(self.meta()),
        ))
    }
}

/// Serialized representation of modifiers / conditions that need runtime lookup.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisteredComponent {
    /// Type name that will be resolved through a registry.
    pub ty: String,
    /// Optional JSON payload interpreted by the runtime.
    #[serde(default)]
    #[cfg_attr(
        not(feature = "binary"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub config: Option<Value>,
}

/// All supported physical input variants.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BindingInputDescriptor {
    /// Keyboard shortcut with zero or more keystrokes (for chords).
    Keyboard {
        /// Sequence of keystrokes for the binding.
        sequence: Vec<Keystroke>,
    },
    /// Multi-keystroke chord (alias of `Keyboard` for clarity).
    KeyboardChord {
        /// Keystroke sequence.
        sequence: Vec<Keystroke>,
    },
    /// Mouse button input.
    MouseButton {
        /// Button identifier (e.g. "left", "right", "middle").
        button: String,
        /// Optional modifier keys required alongside the button.
        #[serde(default)]
        #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
        modifiers: Vec<String>,
    },
    /// Mouse motion.
    MouseMotion {
        #[serde(default)]
        #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
        modifiers: Vec<String>,
    },
    /// Mouse wheel (scroll).
    MouseWheel {
        #[serde(default)]
        #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
        modifiers: Vec<String>,
    },
    /// Gamepad button.
    GamepadButton {
        button: String,
        #[serde(default)]
        #[cfg_attr(
            not(feature = "binary"),
            serde(skip_serializing_if = "Option::is_none")
        )]
        threshold: Option<f32>,
    },
    /// Gamepad axis (stick / trigger).
    GamepadAxis {
        axis: String,
        #[serde(default)]
        #[cfg_attr(
            not(feature = "binary"),
            serde(skip_serializing_if = "Option::is_none")
        )]
        threshold: Option<f32>,
    },
    /// Any key/mouse/gamepad button.
    AnyKey {
        #[serde(default)]
        #[cfg_attr(not(feature = "binary"), serde(skip_serializing_if = "Vec::is_empty"))]
        modifiers: Vec<String>,
    },
    /// Explicitly unbound action.
    None,
}

impl BindingInputDescriptor {
    /// Convenience constructor for keyboard shortcuts.
    pub fn keyboard(sequence: Vec<Keystroke>) -> Self {
        Self::Keyboard { sequence }
    }
}

impl fmt::Display for BindingInputDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyboard { sequence } | Self::KeyboardChord { sequence } => {
                let seq = sequence
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "keyboard({seq})")
            }
            Self::MouseButton { button, .. } => write!(f, "mouse_button({button})"),
            Self::MouseMotion { .. } => write!(f, "mouse_motion"),
            Self::MouseWheel { .. } => write!(f, "mouse_wheel"),
            Self::GamepadButton { button, .. } => write!(f, "gamepad_button({button})"),
            Self::GamepadAxis { axis, .. } => write!(f, "gamepad_axis({axis})"),
            Self::AnyKey { .. } => write!(f, "any_key"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Utility to convert modifier names to [`Modifiers`].
pub fn modifiers_from_strings(names: &[String]) -> Result<Modifiers> {
    let mut modifiers = Modifiers::NONE;
    for name in names {
        match name.as_str() {
            "ctrl" | "control" => {
                modifiers.ctrl = true;
            }
            "alt" | "option" => {
                modifiers.alt = true;
            }
            "shift" => {
                modifiers.shift = true;
            }
            "cmd" | "command" | "super" => {
                modifiers.cmd = true;
            }
            other => anyhow::bail!("unknown modifier '{other}'"),
        }
    }
    Ok(modifiers)
}
