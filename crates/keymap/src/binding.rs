//! Key binding structure and metadata.
//!
//! This module defines the core `KeyBinding` struct that connects keystrokes
//! to actions with optional context predicates.

use crate::action::Action;
use crate::context::KeyBindingContextPredicate;
use crate::keystroke::Keystroke;
use std::sync::Arc;

/// Metadata index for tracking the source of a key binding.
///
/// This is used to implement precedence rules where user bindings
/// take precedence over default bindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyBindingMetaIndex(pub u32);

impl KeyBindingMetaIndex {
    /// User-defined bindings (highest precedence)
    pub const USER: Self = Self(0);
    /// Vim mode bindings
    pub const VIM: Self = Self(1);
    /// Base/plugin bindings
    pub const BASE: Self = Self(2);
    /// Default built-in bindings (lowest precedence)
    pub const DEFAULT: Self = Self(3);
}

/// A key binding that maps keystrokes to an action.
#[derive(Clone)]
pub struct KeyBinding {
    /// The sequence of keystrokes that trigger this binding
    pub keystrokes: Vec<Keystroke>,
    /// The action to execute when triggered
    pub action: Box<dyn Action>,
    /// Optional context predicate that must match for this binding to be active
    pub context_predicate: Option<Arc<KeyBindingContextPredicate>>,
    /// Metadata about the source of this binding (for precedence)
    pub meta: Option<KeyBindingMetaIndex>,
}

impl KeyBinding {
    /// Create a new key binding.
    pub fn new(
        keystrokes: Vec<Keystroke>,
        action: Box<dyn Action>,
        context_predicate: Option<Arc<KeyBindingContextPredicate>>,
    ) -> Self {
        Self {
            keystrokes,
            action,
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

    /// Get the action for this binding.
    pub fn action(&self) -> &dyn Action {
        self.action.as_ref()
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

impl std::fmt::Debug for KeyBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyBinding")
            .field("keystrokes", &self.keystrokes)
            .field("action", &self.action.name())
            .field("context_predicate", &self.context_predicate)
            .field("meta", &self.meta)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action;
    use crate::keystroke::Keystroke;

    action!(TestAction);

    #[test]
    fn test_key_binding_creation() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let action = Box::new(TestAction);
        let binding = KeyBinding::new(keystrokes, action, None);

        assert_eq!(binding.keystrokes.len(), 1);
        assert_eq!(binding.action.name(), "TestAction");
        assert!(binding.context_predicate.is_none());
        assert!(binding.meta.is_none());
    }

    #[test]
    fn test_key_binding_with_meta() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let action = Box::new(TestAction);
        let binding =
            KeyBinding::new(keystrokes, action, None).with_meta(KeyBindingMetaIndex::USER);

        assert_eq!(binding.meta, Some(KeyBindingMetaIndex::USER));
    }

    #[test]
    fn test_match_single_keystroke() {
        let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
        let action = Box::new(TestAction);
        let binding = KeyBinding::new(keystrokes, action, None);

        let typed = vec![Keystroke::parse("cmd-s").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), Some(false)); // Complete match

        let typed = vec![Keystroke::parse("cmd-k").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), None); // No match
    }

    #[test]
    fn test_match_multi_keystroke() {
        let keystrokes = vec![
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-t").unwrap(),
        ];
        let action = Box::new(TestAction);
        let binding = KeyBinding::new(keystrokes, action, None);

        // Partial match
        let typed = vec![Keystroke::parse("cmd-k").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), Some(true));

        // Complete match
        let typed = vec![
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-t").unwrap(),
        ];
        assert_eq!(binding.match_keystrokes(&typed), Some(false));

        // No match - wrong first key
        let typed = vec![Keystroke::parse("cmd-p").unwrap()];
        assert_eq!(binding.match_keystrokes(&typed), None);

        // No match - wrong second key
        let typed = vec![
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-p").unwrap(),
        ];
        assert_eq!(binding.match_keystrokes(&typed), None);

        // No match - too many keys
        let typed = vec![
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-t").unwrap(),
            Keystroke::parse("cmd-p").unwrap(),
        ];
        assert_eq!(binding.match_keystrokes(&typed), None);
    }

    #[test]
    fn test_meta_index_precedence() {
        assert!(KeyBindingMetaIndex::USER < KeyBindingMetaIndex::VIM);
        assert!(KeyBindingMetaIndex::VIM < KeyBindingMetaIndex::BASE);
        assert!(KeyBindingMetaIndex::BASE < KeyBindingMetaIndex::DEFAULT);
    }
}
