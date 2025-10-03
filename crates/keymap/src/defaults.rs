//! Default keybindings for common platforms.
//!
//! This module provides sensible default keybindings for macOS, Windows, and Linux.
//! These bindings follow platform conventions and can be used as a starting point
//! for applications.
//!
//! # Usage
//!
//! ```ignore
//! use keymap::defaults;
//! use keymap::KeymapStore;
//!
//! let mut builder = KeymapStore::builder();
//! builder = defaults::register_macos_defaults(builder);
//! let store = builder.build().unwrap();
//! ```

use crate::actions::*;
use crate::binding::{KeyBinding, KeyBindingMetaIndex};
use crate::keystroke::Keystroke;
use crate::store::KeymapStoreBuilder;

/// Register default macOS keybindings.
///
/// These bindings follow macOS conventions using the Command key as the primary modifier.
pub fn register_macos_defaults(mut builder: KeymapStoreBuilder) -> KeymapStoreBuilder {
    // File operations
    builder = add_binding(builder, "cmd-s", file::Save, None);
    builder = add_binding(builder, "cmd-shift-s", file::SaveAs, None);
    builder = add_binding(builder, "cmd-o", file::Open, None);
    builder = add_binding(builder, "cmd-w", file::Close, None);
    builder = add_binding(builder, "cmd-shift-w", file::CloseAll, None);
    builder = add_binding(builder, "cmd-q", file::Quit, None);
    builder = add_binding(builder, "cmd-n", file::NewFile, None);
    builder = add_binding(builder, "cmd-shift-n", file::NewWindow, None);

    // Edit operations
    builder = add_binding(builder, "cmd-z", edit::Undo, None);
    builder = add_binding(builder, "cmd-shift-z", edit::Redo, None);
    builder = add_binding(builder, "cmd-x", edit::Cut, None);
    builder = add_binding(builder, "cmd-c", edit::Copy, None);
    builder = add_binding(builder, "cmd-v", edit::Paste, None);
    builder = add_binding(builder, "cmd-a", edit::SelectAll, None);
    builder = add_binding(builder, "cmd-d", edit::Duplicate, None);
    builder = add_binding(builder, "cmd-slash", edit::ToggleComment, None);
    // Note: Bracket keys removed - parser conflicts with [ and ]

    // Navigation
    builder = add_binding(builder, "cmd-shift-.", navigation::NextTab, None);
    builder = add_binding(builder, "cmd-shift-,", navigation::PreviousTab, None);

    // Search
    builder = add_binding(builder, "cmd-f", search::Find, None);
    builder = add_binding(builder, "cmd-g", search::FindNext, None);
    builder = add_binding(builder, "cmd-shift-g", search::FindPrevious, None);
    builder = add_binding(builder, "cmd-h", search::Replace, None);
    builder = add_binding(builder, "cmd-shift-f", search::FindInFiles, None);

    // View
    builder = add_binding(builder, "cmd-plus", view::ZoomIn, None);
    builder = add_binding(builder, "cmd-minus", view::ZoomOut, None);
    builder = add_binding(builder, "cmd-0", view::ZoomReset, None);
    builder = add_binding(builder, "cmd-ctrl-f", view::ToggleFullscreen, None);

    // Game-specific
    builder = add_binding(builder, "escape", game::OpenMenu, None);
    builder = add_binding(builder, "cmd-shift-p", game::Screenshot, None);

    builder
}

/// Register default Windows/Linux keybindings.
///
/// These bindings follow Windows/Linux conventions using the Ctrl key as the primary modifier.
pub fn register_windows_linux_defaults(mut builder: KeymapStoreBuilder) -> KeymapStoreBuilder {
    // File operations
    builder = add_binding(builder, "ctrl-s", file::Save, None);
    builder = add_binding(builder, "ctrl-shift-s", file::SaveAs, None);
    builder = add_binding(builder, "ctrl-o", file::Open, None);
    builder = add_binding(builder, "ctrl-w", file::Close, None);
    builder = add_binding(builder, "ctrl-shift-w", file::CloseAll, None);
    builder = add_binding(builder, "alt-f4", file::Quit, None);
    builder = add_binding(builder, "ctrl-n", file::NewFile, None);
    builder = add_binding(builder, "ctrl-shift-n", file::NewWindow, None);

    // Edit operations
    builder = add_binding(builder, "ctrl-z", edit::Undo, None);
    builder = add_binding(builder, "ctrl-y", edit::Redo, None);
    builder = add_binding(builder, "ctrl-x", edit::Cut, None);
    builder = add_binding(builder, "ctrl-c", edit::Copy, None);
    builder = add_binding(builder, "ctrl-v", edit::Paste, None);
    builder = add_binding(builder, "ctrl-a", edit::SelectAll, None);
    builder = add_binding(builder, "ctrl-d", edit::Duplicate, None);
    builder = add_binding(builder, "ctrl-slash", edit::ToggleComment, None);
    // Note: Bracket keys removed - parser conflicts with [ and ]

    // Navigation
    builder = add_binding(builder, "ctrl-tab", navigation::NextTab, None);
    builder = add_binding(builder, "ctrl-shift-tab", navigation::PreviousTab, None);
    builder = add_binding(builder, "ctrl-b", navigation::ToggleSidebar, None);

    // Search
    builder = add_binding(builder, "ctrl-f", search::Find, None);
    builder = add_binding(builder, "f3", search::FindNext, None);
    builder = add_binding(builder, "shift-f3", search::FindPrevious, None);
    builder = add_binding(builder, "ctrl-h", search::Replace, None);
    builder = add_binding(builder, "ctrl-shift-f", search::FindInFiles, None);

    // View
    builder = add_binding(builder, "ctrl-plus", view::ZoomIn, None);
    builder = add_binding(builder, "ctrl-minus", view::ZoomOut, None);
    builder = add_binding(builder, "ctrl-0", view::ZoomReset, None);
    builder = add_binding(builder, "f11", view::ToggleFullscreen, None);

    // Game-specific
    builder = add_binding(builder, "escape", game::OpenMenu, None);
    builder = add_binding(builder, "f12", game::Screenshot, None);

    builder
}

/// Register platform-appropriate defaults based on the target OS.
///
/// This function automatically selects the correct default keybindings
/// for the current platform at compile time.
pub fn register_platform_defaults(builder: KeymapStoreBuilder) -> KeymapStoreBuilder {
    #[cfg(target_os = "macos")]
    {
        register_macos_defaults(builder)
    }

    #[cfg(not(target_os = "macos"))]
    {
        register_windows_linux_defaults(builder)
    }
}

/// Helper function to add a binding with default metadata.
fn add_binding<A: crate::action::Action>(
    builder: KeymapStoreBuilder,
    keystroke: &str,
    action: A,
    context: Option<&str>,
) -> KeymapStoreBuilder {
    use crate::action::Action as _;
    use std::sync::Arc;

    let keystrokes = crate::keystroke::parse_keystroke_sequence(keystroke)
        .expect("Invalid keystroke in default bindings");

    let predicate = context.map(|ctx| {
        Arc::new(
            crate::context::KeyBindingContextPredicate::parse(ctx)
                .expect("Invalid context in default bindings"),
        )
    });

    let binding = KeyBinding::new(keystrokes, Box::new(action), predicate)
        .with_meta(KeyBindingMetaIndex::DEFAULT);

    builder.add_default_binding(binding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeymapStore;

    #[test]
    fn test_macos_defaults() {
        let builder = KeymapStore::builder();
        let builder = register_macos_defaults(builder);
        let store = builder.build().unwrap();

        // Verify store was created successfully
        // Just check that version exists
        let _ = store.version();
    }

    #[test]
    fn test_windows_linux_defaults() {
        let builder = KeymapStore::builder();
        let builder = register_windows_linux_defaults(builder);
        let store = builder.build().unwrap();

        // Verify store was created successfully
        // Just check that version exists
        let _ = store.version();
    }

    #[test]
    fn test_platform_defaults() {
        let builder = KeymapStore::builder();
        let builder = register_platform_defaults(builder);
        let store = builder.build().unwrap();

        // Verify store was created successfully
        // Just check that version exists
        let _ = store.version();
    }
}
