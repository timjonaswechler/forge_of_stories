//! Form field type & metadata (extracted from former `form/mod.rs` – Phase 6.1).
//!
//! This module defines the declarative pieces of the form system:
//! - `FormFieldKind`: Enumeration of supported input widget types
//! - `FormField`: Metadata + optional validator for a single field
//!
//! Responsibilities here are intentionally pure / data-centric. Mutation and
//! interactive behavior live in `popup.rs`; rendering helpers live in `render.rs`.
//!
//! Original behavior preserved – no logic changes were introduced during the split.
//!
//! Usage (unchanged):
//! ```ignore
//! use crate::components::popups::form::{FormField, FormFieldKind};
//!
//! let field = FormField::new("hostname", "Host Name", FormFieldKind::Text)
//!     .help("FQDN or IP the server should bind to")
//!     .validator(|v| {
//!         if v.trim().is_empty() {
//!             Err("Must not be empty".into())
//!         } else {
//!             Ok(())
//!         }
//!     });
//! ```
//!
//! See also:
//! - `schema.rs`  : groups fields into a `FormSchema`
//! - `state.rs`   : mutable runtime editing state
//! - `popup.rs`   : interactive popup implementation
//! - `render.rs`  : rendering & future scrollbar logic

/// A single form field kind supported by the form system.
///
/// Notes:
/// - Text / Secret / Path / Number render as single-line editors
/// - Secret is only obfuscated visually; value kept plain in state
/// - Bool toggles with Left/Right/Space
/// - Select cycles through provided options with Left/Right
/// - ListString holds multiple string entries (e.g. SANs) edited via Insert + Enter
#[derive(Debug, Clone)]
pub enum FormFieldKind {
    Text,
    Secret,
    Path,
    Number,
    Bool,
    Select { options: Vec<String> },
    ListString,
}

/// Declarative description of a form field.
///
/// `validator` (optional):
///   A function receiving the current field value (or each list item for
///   ListString) and returning:
///     Ok(())          -> value accepted
///     Err(message)    -> validation error message (displayed inline)

pub struct FormField {
    pub key: String,
    pub label: String,
    pub kind: FormFieldKind,
    pub help: Option<String>,
    pub validator: Option<Box<dyn Fn(&str) -> std::result::Result<(), String> + Send + Sync>>,
}

impl FormField {
    /// Create a new field definition.
    pub fn new(key: impl Into<String>, label: impl Into<String>, kind: FormFieldKind) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            kind,
            help: None,
            validator: None,
        }
    }

    /// Attach optional help / hint text shown beneath the field.
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Attach a validator closure for the field (textual or list item validation).
    pub fn validator(
        mut self,
        f: impl Fn(&str) -> std::result::Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.validator = Some(Box::new(f));
        self
    }

    /// Return true if this field uses a textual editor when focused.
    pub fn is_textual(&self) -> bool {
        matches!(
            self.kind,
            FormFieldKind::Text
                | FormFieldKind::Secret
                | FormFieldKind::Path
                | FormFieldKind::Number
        )
    }

    /// Return true if this field is a list (multi-value) input.
    pub fn is_list(&self) -> bool {
        matches!(self.kind, FormFieldKind::ListString)
    }
}
