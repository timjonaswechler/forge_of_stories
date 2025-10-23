//! Key binding structure and metadata.
//!
//! This module defines the core [`KeyBinding`] type that connects keystrokes to
//! logical action identifiers with optional context predicates.

use crate::context::KeyBindingContextPredicate;
use crate::keystroke::Keystroke;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::sync::Arc;

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

/// Identifier used to refer to an input context.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContextId(Arc<str>);

impl ContextId {
    /// Create a new context identifier.
    pub fn new(id: impl Into<String>) -> Self {
        let id: String = id.into();
        Self(Arc::<str>::from(id.into_boxed_str()))
    }

    /// Borrow the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ContextId {
    fn from(value: &str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

impl From<String> for ContextId {
    fn from(value: String) -> Self {
        Self(Arc::<str>::from(value.into_boxed_str()))
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for ContextId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ContextId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(ContextId::new(value))
    }
}

/// Metadata index for tracking the source of a key binding.
///
/// This is used to implement precedence rules where user bindings
/// take precedence over default bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeyBindingMetaIndex(pub u32);

impl KeyBindingMetaIndex {
    /// User-defined bindings (highest precedence).
    pub const USER: Self = Self(0);
    /// Vim mode bindings.
    pub const VIM: Self = Self(1);
    /// Base/plugin bindings.
    pub const BASE: Self = Self(2);
    /// Default built-in bindings (lowest precedence).
    pub const DEFAULT: Self = Self(3);
}

impl Default for KeyBindingMetaIndex {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A key binding that maps keystrokes to an action identifier.
#[derive(Clone)]
pub struct KeyBinding {
    /// The sequence of keystrokes that trigger this binding.
    pub keystrokes: Vec<Keystroke>,
    /// Identifier of the action to execute, or `None` when this binding disables the sequence.
    pub action_id: Option<ActionId>,
    /// Optional context predicate that must match for this binding to be active.
    pub context_predicate: Option<Arc<KeyBindingContextPredicate>>,
    /// Metadata about the source of this binding (for precedence).
    pub meta: Option<KeyBindingMetaIndex>,
}

impl KeyBinding {
    /// Create a new key binding.
    pub fn new(
        keystrokes: Vec<Keystroke>,
        action_id: Option<ActionId>,
        context_predicate: Option<Arc<KeyBindingContextPredicate>>,
    ) -> Self {
        Self {
            keystrokes,
            action_id,
            context_predicate,
            meta: None,
        }
    }

    /// Create a key binding with metadata.
    pub fn with_meta(mut self, meta: KeyBindingMetaIndex) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Set the metadata for this binding.
    pub fn set_meta(&mut self, meta: KeyBindingMetaIndex) {
        self.meta = Some(meta);
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

    /// Get the context predicate for this binding.
    pub fn predicate(&self) -> Option<&KeyBindingContextPredicate> {
        self.context_predicate.as_ref().map(|p| p.as_ref())
    }

    /// Get the metadata for this binding.
    pub fn metadata(&self) -> Option<KeyBindingMetaIndex> {
        self.meta
    }
}

impl fmt::Debug for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyBinding")
            .field("keystrokes", &self.keystrokes)
            .field("action_id", &self.action_id)
            .field("context_predicate", &self.context_predicate)
            .field("meta", &self.meta)
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
        let binding = KeyBinding::new(keystrokes, Some(ActionId::from("Save")), None);

        assert_eq!(binding.keystrokes.len(), 1);
        assert_eq!(binding.action_id().unwrap().as_str(), "Save");
        assert!(!binding.is_disabled());
    }

    #[test]
    fn disabled_binding() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let binding = KeyBinding::new(keystrokes, None, None);

        assert!(binding.is_disabled());
    }

    #[test]
    fn match_single_keystroke() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let binding = KeyBinding::new(keystrokes, Some(ActionId::from("Save")), None);

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
        let binding = KeyBinding::new(keystrokes, Some(ActionId::from("OpenThing")), None);

        let typed = vec![Keystroke::parse("cmd-k").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), Some(true));

        let typed = vec![
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-t").unwrap(),
        ];
        assert_eq!(binding.match_keystrokes(&typed), Some(false));
    }
}
