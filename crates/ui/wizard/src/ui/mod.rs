/*!
Root UI module.

Phase 2 (Preparation for larger refactors):
- Introduces a dedicated `ui::keymap` module (moved from former `services::keymap_binding`).
- This `mod` file exists to collect future UI-layer specific submodules (e.g. renderer, layout,
  input adapters, focus management) as the architecture evolves.

Current responsibilities:
- Expose the `keymap` submodule.
- Expose the `render` submodule (Phase 3.3) which contains pure rendering logic.
- Re-export keymap helpers for backwards compatibility.

Migration notes:
- Old imports like `crate::services::keymap_binding::action_from_key` should migrate to either:
    use crate::ui::keymap::action_from_key;
  or rely on the re-exports here:
    use crate::ui::action_from_key;

Future expansion ideas:
- ui::render   (pure rendering helpers)
- ui::focus    (focus ring / traversal logic)
- ui::layout   (higher-level layout strategies)
- ui::intent   (input event → intent normalization layer)
*/

pub mod keymap;
pub mod render; // Phase 3.3: ausgelagerte Rendering-Funktionen

// Re-export commonly used keymap utilities for ergonomic access via `crate::ui::*`.
pub use keymap::{
    action_from_key, chord_from_key, map_label_to_action, mappable_entries_for_context,
};
