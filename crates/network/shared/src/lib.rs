//! Gemeinsame Netzwerktypen für Server- und Client-Crates.
//!
//! Dieses Modul fasst alle wiederverwendbaren Bausteine (IDs, Kanäle, Nachrichten,
//! Serialisierung, Ereignisse, Konfiguration) zusammen. Für bequemen Zugriff
//! sind die wichtigsten Typen über das `prelude`-Modul re-exportiert.

pub mod channels;
pub mod config;
pub mod envelope;
pub mod events;
pub mod ids;
pub mod messages;
pub mod serialization;

pub mod prelude {
    pub use super::channels::*;
    pub use super::config::*;
    pub use super::envelope::*;
    pub use super::events::*;
    pub use super::ids::*;
    pub use super::messages::*;
    pub use super::serialization::*;
}
