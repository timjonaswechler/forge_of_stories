#![allow(dead_code)]

/// Popup components for the Wizard TUI.
///
/// This module only aggregates concrete popup types (alert, confirm, input)
/// and re-exports shared helpers and traits from `components/popup.rs`
/// so there is a single source of truth for popup utilities.
pub mod alert;
pub mod bool_choice;
pub mod confirm;
pub mod form;
pub mod input;
pub mod single_choice;
pub mod keymap;

// Re-export the shared popup helpers and trait from the central popup module
pub use crate::components::popup::{
    PopupComponent, centered_rect_fixed, draw_popup_frame, render_backdrop,
};
pub use crate::components::popups::form::certificate::certificate_wizard_popup;
