//! Form runtime (mutable) state – extracted from former `form/mod.rs` (Phase 6.1).
//!
//! This module contains only the data structures and lightweight helpers
//! representing the *current editing state* of a form:
//!   * Captured scalar values (`values`)
//!   * List (multi-value) inputs (`lists`)
//!   * Validation errors (per-field + global)
//!
//! NO behavioral or semantic changes were introduced in this extraction.
//! Original methods were copied verbatim to keep backwards compatibility.
//!
//! Related modules after the split:
//!   - `field.rs`   : Field definitions (`FormFieldKind`, `FormField`)
//!   - `schema.rs`  : `FormSchema` (declarative configuration)
//!   - `popup.rs`   : Interactive logic (navigation, editing, validation dispatch)
//!   - `render.rs`  : Rendering helpers & (future) pure scrollbar calculations
//!
//! Typical usage:
//! ```ignore
//! let mut state = FormState::default();
//! state.set_value("host", "127.0.0.1");
//! state.set_list("sans", vec!["example.com".into(), "www.example.com".into()]);
//! if let Some(v) = state.get_value("host") {
//!     println!("Host = {v}");
//! }
//! ```
//!
//! The validation process (still implemented in `popup.rs`) mutates `errors`
//! and `global_errors` directly.
//
// NOTE: Keep this module free of UI / rendering concerns to enable future
// serde support or unit tests in isolation.

use std::collections::HashMap;

/// Mutable state captured while editing a form.
///
/// Fields:
/// - `values`: Scalar (stringified) values for textual / numeric / select / bool fields.
///             Bool values are stored as the strings `"true"` or `"false"`.
/// - `lists`:  Multi-value (list) fields keyed by field key.
/// - `errors`: Per-field validation errors (populated during validation phase).
/// - `global_errors`: Cross-field or form-level validation messages.
#[derive(Default, Clone)]
pub struct FormState {
    pub values: HashMap<String, String>,
    pub lists: HashMap<String, Vec<String>>,
    pub errors: HashMap<String, String>,
    pub global_errors: Vec<String>,
}

impl FormState {
    /// Set (or replace) a scalar value for a field.
    pub fn set_value(&mut self, key: &str, value: impl Into<String>) {
        self.values.insert(key.to_string(), value.into());
    }

    /// Get a scalar value for a field (if present).
    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Replace the entire list for a multi-value field.
    pub fn set_list(&mut self, key: &str, items: Vec<String>) {
        self.lists.insert(key.to_string(), items);
    }

    /// Borrow the list slice for a multi-value field.
    pub fn get_list(&self, key: &str) -> Option<&[String]> {
        self.lists.get(key).map(|v| v.as_slice())
    }

    /// Clear all validation artifacts (field + global).
    /// (Not used in original code directly, but a harmless convenience.)
    pub fn clear_validation(&mut self) {
        self.errors.clear();
        self.global_errors.clear();
    }
}
