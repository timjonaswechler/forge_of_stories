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
//! use keymap::{ActionId, KeyBinding, keymap::Keymap, parse_keystroke_sequence};
//!
//! let binding = KeyBinding::new(
//!     parse_keystroke_sequence("cmd-s").unwrap(),
//!     Some(ActionId::from("file::Save")),
//! );
//!
//! let keymap = Keymap::with_bindings(vec![binding]);
//! ```

pub mod binding;
pub mod enhanced;
pub mod keymap;
pub mod keystroke;
pub mod spec;
pub mod store;

// Re-export main types
pub use binding::{ActionId, KeyBinding, KeyBindingMetaIndex};
pub use keymap::{Keymap, KeymapVersion};
pub use keystroke::{Keystroke, Modifiers, parse_keystroke_sequence};
pub use spec::{
    ActionDescriptor, BindingDescriptor, BindingInputDescriptor, KeymapSpec, RegisteredComponent,
};
pub use store::{KeymapFile, KeymapStore, KeymapStoreBuilder};

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
        let binding = KeyBinding::new(vec![keystroke], Some(ActionId::from("Save")));
        assert_eq!(binding.keystrokes.len(), 1);
        assert_eq!(binding.action_id().unwrap().as_str(), "Save");
    }

    #[test]
    fn test_multi_keystroke_sequence() {
        let sequence = parse_keystroke_sequence("cmd-k cmd-t").unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence[0].key, "k");
        assert_eq!(sequence[1].key, "t");
    }
}
