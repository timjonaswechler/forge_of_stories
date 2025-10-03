//! Keymap system for context-aware key binding dispatch.
//!
//! This crate provides a Zed-inspired keymap system with:
//! - Context-based key binding resolution
//! - Multi-keystroke sequences (e.g., "cmd-k cmd-t")
//! - Hierarchical context matching with predicates
//! - Action dispatch system
//! - User/Default binding precedence
//!
//! # Example
//!
//! ```ignore
//! use keymap::{KeyBinding, KeyContext, Keystroke, action};
//!
//! action!(SaveFile);
//!
//! // Create a key binding
//! let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
//! let binding = KeyBinding::new(
//!     keystrokes,
//!     Box::new(SaveFile),
//!     None, // no context restriction
//! );
//!
//! // Create a context
//! let mut context = KeyContext::default();
//! context.add("Editor");
//! ```

pub mod action;
pub mod binding;
pub mod context;
pub mod keymap;
pub mod keystroke;
pub mod store;

// Bevy integration (optional)
#[cfg(feature = "bevy_plugin")]
pub mod bevy;

// Re-export main types
pub use action::{Action, NoAction, is_no_action};
pub use binding::{KeyBinding, KeyBindingMetaIndex};
pub use context::{ContextEntry, KeyBindingContextPredicate, KeyContext};
pub use keymap::{Keymap, KeymapVersion};
pub use keystroke::{Keystroke, Modifiers, parse_keystroke_sequence};
pub use store::{KeymapFile, KeymapSection, KeymapStore, KeymapStoreBuilder};

// Re-export macros
pub use action as action_macro;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    action!(TestAction);

    #[test]
    fn test_basic_workflow() {
        // Create a keystroke
        let keystroke = Keystroke::parse("cmd-s").unwrap();
        assert_eq!(keystroke.key, "s");
        assert!(keystroke.modifiers.cmd);

        // Create an action
        let action = Box::new(TestAction);
        assert_eq!(action.name(), "TestAction");

        // Create a binding
        let binding = KeyBinding::new(vec![keystroke], action, None);
        assert_eq!(binding.keystrokes.len(), 1);

        // Create a context
        let mut context = KeyContext::default();
        context.add("Editor");
        assert!(context.contains("Editor"));
    }

    #[test]
    fn test_context_predicate() {
        let predicate = KeyBindingContextPredicate::parse("Editor && mode == full").unwrap();

        let mut context = KeyContext::default();
        context.add("Editor");
        context.set("mode", "full");

        assert!(predicate.eval(&[context.clone()]));

        let mut wrong_context = KeyContext::default();
        wrong_context.add("Editor");
        wrong_context.set("mode", "minimal");

        assert!(!predicate.eval(&[wrong_context]));
    }

    #[test]
    fn test_multi_keystroke_sequence() {
        let sequence = parse_keystroke_sequence("cmd-k cmd-t").unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence[0].key, "k");
        assert_eq!(sequence[1].key, "t");
    }
}
