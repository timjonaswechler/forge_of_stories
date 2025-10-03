//! Default actions for common operations.
//!
//! This module provides a standard set of actions that can be used across
//! all UIs (illusion, fate, wizard). These actions represent common operations
//! and provide a consistent naming scheme.
//!
//! # Categories
//!
//! - **File**: Save, Open, Close, Quit operations
//! - **Edit**: Undo, Redo, Cut, Copy, Paste operations
//! - **Navigation**: Tab navigation, search operations
//! - **View**: Zoom, panels, toggles
//!
//! # Usage
//!
//! ```ignore
//! use keymap::actions::file::*;
//! use keymap::actions::edit::*;
//!
//! // Register actions with the store
//! store.register_action("file::Save", || Box::new(Save));
//! store.register_action("edit::Copy", || Box::new(Copy));
//! ```

use crate::action;
use crate::action_with_data;

/// File operations
pub mod file {
    use super::*;

    action!(Save);
    action!(SaveAs);
    action!(SaveAll);
    action!(Open);
    action!(OpenRecent);
    action!(Close);
    action!(CloseAll);
    action!(CloseOthers);
    action!(Revert);
    action!(Quit);
    action!(NewFile);
    action!(NewWindow);
}

/// Edit operations
pub mod edit {
    use super::*;

    action!(Undo);
    action!(Redo);
    action!(Cut);
    action!(Copy);
    action!(Paste);
    action!(SelectAll);
    action!(Delete);
    action!(Duplicate);
    action!(MoveLineUp);
    action!(MoveLineDown);
    action!(ToggleComment);
    action!(IndentMore);
    action!(IndentLess);
    action!(JoinLines);
}

/// Navigation operations
pub mod navigation {
    use super::*;

    action!(NextTab);
    action!(PreviousTab);
    action!(NextPane);
    action!(PreviousPane);
    action!(FocusEditor);
    action!(FocusTerminal);
    action!(ToggleSidebar);
    action!(ToggleBottomPanel);

    action_with_data!(GoToLine { line: usize });

    action_with_data!(GoToDefinition { symbol: String });

    action_with_data!(FindInFile { pattern: String });

    action_with_data!(ReplaceInFile {
        pattern: String,
        replacement: String
    });
}

/// View operations
pub mod view {
    use super::*;

    action!(ZoomIn);
    action!(ZoomOut);
    action!(ZoomReset);
    action!(ToggleFullscreen);
    action!(ToggleZenMode);
    action!(SplitHorizontal);
    action!(SplitVertical);
    action!(ClosePane);
    action!(ToggleLineNumbers);
    action!(ToggleMinimap);
}

/// Search operations
pub mod search {
    use super::*;

    action!(Find);
    action!(FindNext);
    action!(FindPrevious);
    action!(Replace);
    action!(ReplaceAll);
    action!(FindInFiles);
    action!(ReplaceInFiles);
    action!(ClearSearch);
}

/// Debug operations (for development/debugging UI)
pub mod debug {
    use super::*;

    action!(ToggleInspector);
    action!(ToggleDebugOverlay);
    action!(ReloadKeymap);
    action!(DumpState);
    action!(TogglePerformanceOverlay);
}

/// Game-specific operations
pub mod game {
    use super::*;

    action!(Pause);
    action!(Resume);
    action!(QuickSave);
    action!(QuickLoad);
    action!(OpenMenu);
    action!(OpenInventory);
    action!(OpenMap);
    action!(ToggleConsole);
    action!(Screenshot);
}

/// Helper function to register all default actions with a store builder
///
/// # Example
///
/// ```ignore
/// use keymap::KeymapStore;
/// use keymap::actions::register_default_actions;
///
/// let mut builder = KeymapStore::builder();
/// builder = register_default_actions(builder);
/// let store = builder.build().unwrap();
/// ```
#[cfg(feature = "bevy_plugin")]
pub fn register_default_actions(
    mut builder: crate::store::KeymapStoreBuilder,
) -> crate::store::KeymapStoreBuilder {
    use crate::action::Action;

    // File actions
    builder = builder.register_action("file::Save", || Box::new(file::Save) as Box<dyn Action>);
    builder = builder.register_action("file::SaveAs", || Box::new(file::SaveAs) as Box<dyn Action>);
    builder = builder.register_action("file::SaveAll", || {
        Box::new(file::SaveAll) as Box<dyn Action>
    });
    builder = builder.register_action("file::Open", || Box::new(file::Open) as Box<dyn Action>);
    builder = builder.register_action("file::OpenRecent", || {
        Box::new(file::OpenRecent) as Box<dyn Action>
    });
    builder = builder.register_action("file::Close", || Box::new(file::Close) as Box<dyn Action>);
    builder = builder.register_action("file::CloseAll", || {
        Box::new(file::CloseAll) as Box<dyn Action>
    });
    builder = builder.register_action("file::CloseOthers", || {
        Box::new(file::CloseOthers) as Box<dyn Action>
    });
    builder = builder.register_action("file::Revert", || Box::new(file::Revert) as Box<dyn Action>);
    builder = builder.register_action("file::Quit", || Box::new(file::Quit) as Box<dyn Action>);
    builder = builder.register_action("file::NewFile", || {
        Box::new(file::NewFile) as Box<dyn Action>
    });
    builder = builder.register_action("file::NewWindow", || {
        Box::new(file::NewWindow) as Box<dyn Action>
    });

    // Edit actions
    builder = builder.register_action("edit::Undo", || Box::new(edit::Undo) as Box<dyn Action>);
    builder = builder.register_action("edit::Redo", || Box::new(edit::Redo) as Box<dyn Action>);
    builder = builder.register_action("edit::Cut", || Box::new(edit::Cut) as Box<dyn Action>);
    builder = builder.register_action("edit::Copy", || Box::new(edit::Copy) as Box<dyn Action>);
    builder = builder.register_action("edit::Paste", || Box::new(edit::Paste) as Box<dyn Action>);
    builder = builder.register_action("edit::SelectAll", || {
        Box::new(edit::SelectAll) as Box<dyn Action>
    });
    builder = builder.register_action("edit::Delete", || Box::new(edit::Delete) as Box<dyn Action>);
    builder = builder.register_action("edit::Duplicate", || {
        Box::new(edit::Duplicate) as Box<dyn Action>
    });
    builder = builder.register_action("edit::MoveLineUp", || {
        Box::new(edit::MoveLineUp) as Box<dyn Action>
    });
    builder = builder.register_action("edit::MoveLineDown", || {
        Box::new(edit::MoveLineDown) as Box<dyn Action>
    });
    builder = builder.register_action("edit::ToggleComment", || {
        Box::new(edit::ToggleComment) as Box<dyn Action>
    });
    builder = builder.register_action("edit::IndentMore", || {
        Box::new(edit::IndentMore) as Box<dyn Action>
    });
    builder = builder.register_action("edit::IndentLess", || {
        Box::new(edit::IndentLess) as Box<dyn Action>
    });
    builder = builder.register_action("edit::JoinLines", || {
        Box::new(edit::JoinLines) as Box<dyn Action>
    });

    // Navigation actions
    builder = builder.register_action("navigation::NextTab", || {
        Box::new(navigation::NextTab) as Box<dyn Action>
    });
    builder = builder.register_action("navigation::PreviousTab", || {
        Box::new(navigation::PreviousTab) as Box<dyn Action>
    });
    builder = builder.register_action("navigation::NextPane", || {
        Box::new(navigation::NextPane) as Box<dyn Action>
    });
    builder = builder.register_action("navigation::PreviousPane", || {
        Box::new(navigation::PreviousPane) as Box<dyn Action>
    });
    builder = builder.register_action("navigation::FocusEditor", || {
        Box::new(navigation::FocusEditor) as Box<dyn Action>
    });
    builder = builder.register_action("navigation::FocusTerminal", || {
        Box::new(navigation::FocusTerminal) as Box<dyn Action>
    });
    builder = builder.register_action("navigation::ToggleSidebar", || {
        Box::new(navigation::ToggleSidebar) as Box<dyn Action>
    });
    builder = builder.register_action("navigation::ToggleBottomPanel", || {
        Box::new(navigation::ToggleBottomPanel) as Box<dyn Action>
    });

    // View actions
    builder = builder.register_action("view::ZoomIn", || Box::new(view::ZoomIn) as Box<dyn Action>);
    builder = builder.register_action("view::ZoomOut", || {
        Box::new(view::ZoomOut) as Box<dyn Action>
    });
    builder = builder.register_action("view::ZoomReset", || {
        Box::new(view::ZoomReset) as Box<dyn Action>
    });
    builder = builder.register_action("view::ToggleFullscreen", || {
        Box::new(view::ToggleFullscreen) as Box<dyn Action>
    });
    builder = builder.register_action("view::ToggleZenMode", || {
        Box::new(view::ToggleZenMode) as Box<dyn Action>
    });
    builder = builder.register_action("view::SplitHorizontal", || {
        Box::new(view::SplitHorizontal) as Box<dyn Action>
    });
    builder = builder.register_action("view::SplitVertical", || {
        Box::new(view::SplitVertical) as Box<dyn Action>
    });
    builder = builder.register_action("view::ClosePane", || {
        Box::new(view::ClosePane) as Box<dyn Action>
    });
    builder = builder.register_action("view::ToggleLineNumbers", || {
        Box::new(view::ToggleLineNumbers) as Box<dyn Action>
    });
    builder = builder.register_action("view::ToggleMinimap", || {
        Box::new(view::ToggleMinimap) as Box<dyn Action>
    });

    // Search actions
    builder = builder.register_action("search::Find", || Box::new(search::Find) as Box<dyn Action>);
    builder = builder.register_action("search::FindNext", || {
        Box::new(search::FindNext) as Box<dyn Action>
    });
    builder = builder.register_action("search::FindPrevious", || {
        Box::new(search::FindPrevious) as Box<dyn Action>
    });
    builder = builder.register_action("search::Replace", || {
        Box::new(search::Replace) as Box<dyn Action>
    });
    builder = builder.register_action("search::ReplaceAll", || {
        Box::new(search::ReplaceAll) as Box<dyn Action>
    });
    builder = builder.register_action("search::FindInFiles", || {
        Box::new(search::FindInFiles) as Box<dyn Action>
    });
    builder = builder.register_action("search::ReplaceInFiles", || {
        Box::new(search::ReplaceInFiles) as Box<dyn Action>
    });
    builder = builder.register_action("search::ClearSearch", || {
        Box::new(search::ClearSearch) as Box<dyn Action>
    });

    // Debug actions
    builder = builder.register_action("debug::ToggleInspector", || {
        Box::new(debug::ToggleInspector) as Box<dyn Action>
    });
    builder = builder.register_action("debug::ToggleDebugOverlay", || {
        Box::new(debug::ToggleDebugOverlay) as Box<dyn Action>
    });
    builder = builder.register_action("debug::ReloadKeymap", || {
        Box::new(debug::ReloadKeymap) as Box<dyn Action>
    });
    builder = builder.register_action("debug::DumpState", || {
        Box::new(debug::DumpState) as Box<dyn Action>
    });
    builder = builder.register_action("debug::TogglePerformanceOverlay", || {
        Box::new(debug::TogglePerformanceOverlay) as Box<dyn Action>
    });

    // Game actions
    builder = builder.register_action("game::Pause", || Box::new(game::Pause) as Box<dyn Action>);
    builder = builder.register_action("game::Resume", || Box::new(game::Resume) as Box<dyn Action>);
    builder = builder.register_action("game::QuickSave", || {
        Box::new(game::QuickSave) as Box<dyn Action>
    });
    builder = builder.register_action("game::QuickLoad", || {
        Box::new(game::QuickLoad) as Box<dyn Action>
    });
    builder = builder.register_action("game::OpenMenu", || {
        Box::new(game::OpenMenu) as Box<dyn Action>
    });
    builder = builder.register_action("game::OpenInventory", || {
        Box::new(game::OpenInventory) as Box<dyn Action>
    });
    builder = builder.register_action("game::OpenMap", || {
        Box::new(game::OpenMap) as Box<dyn Action>
    });
    builder = builder.register_action("game::ToggleConsole", || {
        Box::new(game::ToggleConsole) as Box<dyn Action>
    });
    builder = builder.register_action("game::Screenshot", || {
        Box::new(game::Screenshot) as Box<dyn Action>
    });

    builder
}
