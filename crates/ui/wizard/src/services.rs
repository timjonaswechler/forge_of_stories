// Phase 2 refactor: keymap binding logic moved to `ui::keymap`.
// Re-export here to avoid breaking existing (yet-to-be-updated) imports that used `crate::services::keymap_binding::*`.
pub use crate::ui::keymap::{
    action_from_key, chord_from_key, map_label_to_action, mappable_entries_for_context,
};
