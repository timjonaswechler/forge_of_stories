#![allow(missing_docs)]
// Domain layer modules.
//
// This module contains non-UI (domain) logic that the TUI consumes as data.
// Keeping UI (components/pages) separate from domain logic improves testability
// and prepares the codebase for a cleaner architecture (reducers, effects, etc).
//
// Phase 8 extraction:
// - preflight: environment / system checks (moved out of welcome component)
// - settings_gateway: facade around settings + aether_config interaction
// - certs: placeholder for future certificate generation utilities
//
// Add future domain modules here (e.g. persistence, reducers, async services).

pub mod certs;
pub mod preflight;
pub mod settings_gateway;
