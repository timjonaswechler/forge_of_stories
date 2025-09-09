//! Form subsystem (Phase 6 – structural split)
//!
//! This directory used to be a single large `form/mod.rs` file containing:
//! - Field kind definitions
//! - Field metadata struct
//! - Schema struct
//! - Mutable runtime state
//! - Popup implementation (event handling, validation, editing)
//! - Rendering (including scrollbar calculations)
//!
//! Phase 6.1 goal: Pure structural refactor without logic changes.
//! The original code has been moved into the following modules:
//!
//! Modules:
//! - `field.rs`   : `FormFieldKind`, `FormField` (definitions + helpers)
//! - `schema.rs`  : `FormSchema` (declarative description)
//! - `state.rs`   : `FormState` (mutable editing state + helpers)
//! - `popup.rs`   : `FormPopup` (interaction + navigation + validation + submit building)
//! - `render.rs`  : Rendering helpers (pure / side-effect free where possible)
//! - `certificate` (pre-existing domain-specific form builders)
//!
//! Re‑exports below preserve the previous public surface so existing imports like:
//!     use crate::components::popups::form::{FormPopup, FormSchema, FormField, FormFieldKind};
//! continue to work unchanged.
//!
//! Follow-ups (later Phase 6 tasks):
//! - 6.2: Add unit tests for validators (country code, SAN list item, number range).
//! - 6.3: Extract scrollbar/thumb computation into a pure function inside `render.rs`
//!        and unit test it.
//!
//! IMPORTANT: No behavioral changes were introduced in 6.1; only a file layout refactor.

// Domain-specific certificate-related form builders (unchanged)
pub mod certificate;

// Internal modules (implementation detail of the form system)
pub mod field;
pub mod popup;
pub mod render;
pub mod schema;
pub mod state;

// Public re-exports (stable API surface)
pub use field::{FormField, FormFieldKind};
pub use popup::FormPopup;
/* (Phase 6.1)
   Temporarily removing re-exports of render helpers to avoid unused warnings.
   They will be reintroduced in Task 6.3 once scrollbar logic is finalized and
   call sites begin using them explicitly.
*/
pub use schema::FormSchema;
pub use state::FormState;

// (Phase 6.1) Removed unused `prelude` module (was generating unused-public-items warnings).

// Backwards compatibility note:
// If legacy code still expects methods that lived on FormPopup tied to rendering,
// they are now thin wrappers delegating into `render` (added in popup.rs during split).
//
// Any new logic SHOULD prefer placing:
// - Pure calculations -> `render.rs` / dedicated helper modules
// - State mutation / navigation -> `popup.rs`
// - Type definitions / metadata -> `field.rs`, `schema.rs`, `state.rs`
