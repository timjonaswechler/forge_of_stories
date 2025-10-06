//! Embedded server module for client-hosted gameplay.

pub mod embedded;

pub use embedded::{EmbeddedServer, ServerError, ServerMode, ServerState};
