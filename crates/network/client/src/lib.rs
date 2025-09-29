//! Clientseitige Netzwerkkomponenten und Bevy-Integration.
//!
//! Stellt Konfiguration, Transport (QUIC, später Steam), Discovery,
//! Verbindungsverwaltung, Synchronisation und Telemetrie bereit.

pub mod config;
pub mod connection;
pub mod discovery;
pub mod error;
pub mod events;
pub mod metrics;
pub mod runtime;
pub mod sync;
pub mod transport;
