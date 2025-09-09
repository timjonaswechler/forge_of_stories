//! Transitional intent module.
//!
//! Phase 2 Task 2.1:
//! This module introduces the `Intent` alias for the existing `Action` enum.
//! During the refactor we will gradually migrate call sites from `Action` to `Intent`
//! without breaking existing code. Later phases can then:
//!   1. Rename or split semantics (e.g. distinguish UI intents vs. system effects).
//!   2. Introduce a reducer pattern operating on `Intent`.
//!
//! Migration strategy:
//!   - Step 1 (now): `pub use Action as Intent;`
//!   - Step 2: Update new code to prefer `Intent`.
//!   - Step 3: Optionally rename original enum or redefine a slimmer `Action` for rendering loop.
//!
//! Nothing else changes functionally at this point.

use crate::action::Action;

// Public re-export: existing `Action` is now also addressable as `Intent`.
pub use Action as Intent;
