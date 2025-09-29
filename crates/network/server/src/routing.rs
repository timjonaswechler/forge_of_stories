//! Weiterleitung von Transportereignissen in die Gameplay-Systeme.
//!
//! Geplante API:
//! - Dispatcher, der `TransportEvent` in spezifische `NetworkEvent`s überführt.
//! - Broadcast-/Unicast-Helfer für Antworten an Clients.
//! - Prioritätssteuerung basierend auf Kanaldefinitionen.
