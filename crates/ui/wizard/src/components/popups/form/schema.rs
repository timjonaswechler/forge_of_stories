//! Form schema definition (extracted from former `form/mod.rs` – Phase 6.1).
//!
//! This module provides the declarative container `FormSchema` which groups multiple
//! `FormField` instances together along with presentation metadata (title, description,
//! sizing hints).
//!
//! Split rationale:
//! - Previously `FormSchema` lived alongside state, rendering, and popup interaction
//!   code in a single large file. Moving it here keeps the data model clean and
//!   dependency‑light.
//! - Keeping schema pure (no mutation logic beyond simple builder setters) simplifies
//!   future serialization or configuration driven form generation.
//!
//! No behavioral changes were introduced during this extraction.
//!
//! Related modules:
//! - `field.rs`  : `FormFieldKind`, `FormField`
//! - `state.rs`  : `FormState` (mutable runtime values + validation errors)
//! - `popup.rs`  : `FormPopup` (interactive behavior / event handling)
//! - `render.rs` : Rendering helpers & (future) scrollbar calculations
//!
//! Typical usage (unchanged):
//! ```ignore
//! use crate::components::popups::form::{FormSchema, FormField, FormFieldKind};
//!
//! let schema = FormSchema::new("Server Setup", vec![
//!     FormField::new("host", "Host", FormFieldKind::Text)
//!         .help("Hostname or IP the server should bind to"),
//!     FormField::new("port", "Port", FormFieldKind::Number)
//!         .validator(|v| v.parse::<u16>()
//!             .map(|_| ()).map_err(|_| "Must be a valid u16 port".into()))
//! ]).description("Configure basic network parameters");
//! ```
use super::FormField;

/// Declarative schema for a multi‑field form.
///
/// Fields:
/// - `title`:        Display title in the popup frame
/// - `description`:  Optional descriptive text rendered above the fields
/// - `fields`:       Ordered collection of `FormField` definitions
/// - `min_width` / `min_height`: Layout hints used by the popup renderer
///
/// NOTE: Kept intentionally lightweight; validation & transformation rules
/// remain attached to each `FormField` (via its optional validator closure).
pub struct FormSchema {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FormField>,
    pub min_width: u16,
    pub min_height: u16,
}

impl FormSchema {
    /// Create a new schema with default sizing (width=60, height=16) and no description.
    pub fn new(title: impl Into<String>, fields: Vec<FormField>) -> Self {
        Self {
            title: title.into(),
            description: None,
            fields,
            min_width: 60,
            min_height: 16,
        }
    }

    /// Attach an optional description (multi‑line friendly).
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Override minimum size hints (clamped to sensible lower bounds).
    pub fn min_size(mut self, w: u16, h: u16) -> Self {
        self.min_width = w.max(40);
        self.min_height = h.max(10);
        self
    }

    /// Convenience accessor: number of fields.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Find a field by its key.
    pub fn field_by_key(&self, key: &str) -> Option<&FormField> {
        self.fields.iter().find(|f| f.key == key)
    }
}
