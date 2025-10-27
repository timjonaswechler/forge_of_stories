//! Key binding structure and metadata.
//!
//! This module defines the core [`KeyBinding`] type that connects keystrokes to
//! logical action identifiers with optional context predicates.

use crate::keystroke::Keystroke;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::sync::Arc;

/// Metadata index for key binding precedence.
///
/// This determines the priority when multiple bindings match the same keystroke.
/// Lower values have higher precedence (are checked first).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBindingMetaIndex(u32);

impl KeyBindingMetaIndex {
    /// Default bindings (lowest precedence).
    pub const DEFAULT: Self = Self(100);

    /// User-defined bindings (highest precedence).
    pub const USER: Self = Self(0);

    /// Create a new metadata index with custom precedence.
    pub const fn new(precedence: u32) -> Self {
        Self(precedence)
    }
}

/// Identifier used to refer to a logical action.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionId(Arc<str>);

impl ActionId {
    /// Create a new action identifier.
    pub fn new(id: impl Into<String>) -> Self {
        let id: String = id.into();
        Self(Arc::<str>::from(id.into_boxed_str()))
    }

    /// Borrow the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ActionId {
    fn from(value: &str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

impl From<String> for ActionId {
    fn from(value: String) -> Self {
        Self(Arc::<str>::from(value.into_boxed_str()))
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for ActionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ActionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(ActionId::new(value))
    }
}

/// A key binding that maps keystrokes to an action identifier.
#[derive(Clone)]
pub struct KeyBinding {
    /// The sequence of keystrokes that trigger this binding.
    pub keystrokes: Vec<Keystroke>,
    /// Identifier of the action to execute, or `None` when this binding disables the sequence.
    pub action_id: Option<ActionId>,
    /// Metadata for binding precedence.
    meta: Option<KeyBindingMetaIndex>,
}

impl KeyBinding {
    /// Create a new key binding.
    pub fn new(keystrokes: Vec<Keystroke>, action_id: Option<ActionId>) -> Self {
        Self {
            keystrokes,
            action_id,
            meta: None,
        }
    }

    /// Returns `true` if this binding disables the associated keystroke sequence.
    pub fn is_disabled(&self) -> bool {
        self.action_id.is_none()
    }

    /// Check if the given keystrokes match this binding.
    ///
    /// Returns:
    /// - `None` if the keystrokes don't match at all
    /// - `Some(true)` if the keystrokes match partially (more keystrokes needed)
    /// - `Some(false)` if the keystrokes match completely
    pub fn match_keystrokes(&self, typed: &[Keystroke]) -> Option<bool> {
        if self.keystrokes.len() < typed.len() {
            return None;
        }

        for (target, typed) in self.keystrokes.iter().zip(typed.iter()) {
            if !target.matches(typed) {
                return None;
            }
        }

        Some(self.keystrokes.len() > typed.len())
    }

    /// Get the keystrokes for this binding.
    pub fn keystrokes(&self) -> &[Keystroke] {
        &self.keystrokes
    }

    /// Get the action identifier for this binding.
    pub fn action_id(&self) -> Option<&ActionId> {
        self.action_id.as_ref()
    }

    /// Get the metadata index for this binding.
    pub fn metadata(&self) -> Option<KeyBindingMetaIndex> {
        self.meta
    }

    /// Set the metadata index for this binding.
    pub fn set_meta(&mut self, meta: KeyBindingMetaIndex) {
        self.meta = Some(meta);
    }
}

impl fmt::Debug for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyBinding")
            .field("keystrokes", &self.keystrokes)
            .field("action_id", &self.action_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystroke::Keystroke;

    #[test]
    fn action_id_conversions() {
        let id = ActionId::new("TestAction");
        assert_eq!(id.as_str(), "TestAction");

        let from_string = ActionId::from(String::from("Another"));
        assert_eq!(from_string.as_str(), "Another");
    }

    #[test]
    fn key_binding_creation() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let binding = KeyBinding::new(keystrokes, Some(ActionId::from("Save")));

        assert_eq!(binding.keystrokes.len(), 1);
        assert_eq!(binding.action_id().unwrap().as_str(), "Save");
        assert!(!binding.is_disabled());
    }

    #[test]
    fn disabled_binding() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let binding = KeyBinding::new(keystrokes, None);

        assert!(binding.is_disabled());
    }

    #[test]
    fn match_single_keystroke() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let binding = KeyBinding::new(keystrokes, Some(ActionId::from("Save")));

        let typed = vec![Keystroke::parse("cmd-s").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), Some(false));

        let typed = vec![Keystroke::parse("cmd-k").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), None);
    }

    #[test]
    fn match_multi_keystroke() {
        let keystrokes = vec![
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-t").unwrap(),
        ];
        let binding = KeyBinding::new(keystrokes, Some(ActionId::from("OpenThing")));

        let typed = vec![Keystroke::parse("cmd-k").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), Some(true));

        let typed = vec![
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-t").unwrap(),
        ];
        assert_eq!(binding.match_keystrokes(&typed), Some(false));
    }
}
