//! Keymap system for context-aware key binding dispatch.
//!
//! The crate focuses on *data* handling — defining, merging, and persisting
//! key bindings that can later be applied to input runtimes such as
//! `bevy_enhanced_input`. Default bindings can be provided in code, user
//! overrides are stored in JSON, and the merged result is exposed through a
//! deterministic matching engine.
//!
//! # Example
//!
//! ```ignore
//! use keymap::{ActionId, KeyBinding, KeyContext, parse_keystroke_sequence};
//!
//! let binding = KeyBinding::new(
//!     parse_keystroke_sequence("cmd-s").unwrap(),
//!     Some(ActionId::from("file::Save")),
//!     None, // no context restriction
//! );
//!
//! let mut context = KeyContext::default();
//! context.add("Editor");
//! ```

pub mod binding;
pub mod context;
pub mod keymap;
pub mod keystroke;
pub mod store;

pub mod enhanced;
pub mod spec;

// Re-export main types
pub use binding::{ActionId, ContextId, KeyBinding, KeyBindingMetaIndex};
pub use context::{ContextEntry, KeyBindingContextPredicate, KeyContext};
pub use keymap::{Keymap, KeymapVersion};
pub use keystroke::{Keystroke, Modifiers, parse_keystroke_sequence};
pub use store::{KeymapFile, KeymapStore, KeymapStoreBuilder};
pub use spec::{
    ActionDescriptor, BindingDescriptor, BindingInputDescriptor, ContextDescriptor, KeymapSpec,
    RegisteredComponent,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_workflow() {
        // Create a keystroke
        let keystroke = Keystroke::parse("cmd-s").unwrap();
        assert_eq!(keystroke.key, "s");
        assert!(keystroke.modifiers.cmd);

        // Create a binding
        let binding = KeyBinding::new(vec![keystroke], Some(ActionId::from("Save")), None);
        assert_eq!(binding.keystrokes.len(), 1);
        assert_eq!(binding.action_id().unwrap().as_str(), "Save");

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
