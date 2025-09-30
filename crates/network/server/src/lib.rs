//! Serverseitige Netzwerk-Laufzeit und Bevy-Integration.
//!
//! Exportiert Konfiguration, Transport-Implementierungen, Discovery, Authentifizierung,
//! Sitzungsverwaltung sowie die Bevy-Plugin-Anbindung.

pub mod auth;
pub mod config;
pub mod discovery;
pub mod error;
pub mod metrics;
pub mod plugin;
pub mod routing;
pub mod runtime;
pub mod session;
pub mod transport;
pub mod steam;
