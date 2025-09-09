#![allow(dead_code)]
//! Transport Abstraction Layer
//!
//! Ziel:
//! - Einheitliche Schnittstelle für verschiedene Netzwerk-Transports (QUIC, Steam Relay, Local/InProc).
//! - Minimiert Kopplung zwischen niedriger Transport-Ebene und Protokoll-/Session-Layer.
//!
//! Design-Prinzipien:
//! - Provider liefern rohe Events (Verbindungen / rohe Bytes) ohne Protokoll-Interpretation.
//! - Handshake, Framing, Sessions, Rate-Limits passieren eine Ebene darüber.
//! - Mehrere Provider können parallel aktiv sein (z. B. Hybrid: Steam + QUIC).
//!
//! Lebenszyklus:
//! 1. `start(cfg)` jedes Providers → interner Listener / Endpoint init
//! 2. Bevy-System ruft pro Tick `poll_events()` auf → sammelt ProviderEvent in temporären Buffer
//! 3. Höhere Schicht verarbeitet Buffer (Handshake / Frame Decode / Session Mgmt)
//! 4. Senden / Disconnect über `send` / `disconnect`
//! 5. Shutdown (App-Ende) über `shutdown` (Plugin Drop oder explizit)
//!
//! Threading / Async:
//! - Provider dürfen intern Tokio-Tasks starten.
//! - `poll_events` MUSS non-blocking sein und keine heavy Arbeit verrichten.
//!
//! Erweitern:
//! - Neuen Provider anlegen (z. B. `steam_provider.rs`) → `impl TransportProvider`.
//! - In `ActiveTransports` registrieren (Plugin Startup).
//!
//! Wichtige Folgeschichten (nicht hier implementiert):
//! - Handshake Pipeline
//! - Frame Codec
//! - Session Registry
//! - Metrics / Rate Limits
//!
//! (C) Forge of Stories – Netzwerk-Architektur Evolution (M1)

use std::{
    any::Any,
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex},
};

use crate::ServerRuntimeConfig;

pub mod quic_provider;
pub mod steam_provider; // Steam Relay (Stub) – registrierbar via Mode-Dispatch

/// TransportEventBuffer:
/// Frame-übergreifender Buffer für ProviderEvents um:
/// - genau ein Poll der Provider pro Frame zu erzwingen
/// - spätere Systeme (Handshake / Session Router / PingPong / Metrics) von
///   direktem ActiveTransports-Zugriff zu entkoppeln.
/// Der Buffer wird von einem zentralen Poll-System gefüllt und danach von
/// nachfolgenden Systemen geleert / verarbeitet.
#[derive(bevy::prelude::Resource, Default)]
pub struct TransportEventBuffer {
    pub events: Vec<ProviderEvent>,
}

impl TransportEventBuffer {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Hängt Events an (verwendet durch das Poll-System).
    pub fn extend<I: IntoIterator<Item = ProviderEvent>>(&mut self, it: I) {
        self.events.extend(it);
    }

    /// Nimmt alle Events heraus (Ownership Transfer) – danach leer.
    pub fn take(&mut self) -> Vec<ProviderEvent> {
        std::mem::take(&mut self.events)
    }

    /// Liefert aktuelle Anzahl gepufferter Events (Diagnose).
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

use bevy::prelude::Resource;

/// Eindeutige ID für eine logische Transport-Verbindung (Provider-agnostisch).
/// Intern monotone Zähler (Vergabe im jeweiligen Provider).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConnectionId(pub u64);

impl fmt::Debug for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Conn({})", self.0)
    }
}

/// Typ des Providers – zur Diagnose / Metrik / Routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Local,
    Quic,
    Steam,
    Custom(&'static str),
}

/// Ereignisse vom Transport bevor Protokoll / Handshake stattfindet.
/// Reihenfolge wird pro Provider beibehalten, zwischen Providern nicht garantiert.
#[derive(Debug)]
pub enum ProviderEvent {
    NewConnection {
        id: ConnectionId,
        remote: String,
        via: ProviderKind,
    },
    Disconnected {
        id: ConnectionId,
        reason: Option<String>,
        via: ProviderKind,
    },
    /// Rohdaten (z. B. erstes Handshake-Frame) – NICHT decodiert.
    RawInbound {
        id: ConnectionId,
        bytes: Vec<u8>,
        via: ProviderKind,
    },
}

/// Fehlerart für generische Transportoperationen.
#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    #[error("unknown connection")]
    UnknownConnection,
    #[error("send failed: {0}")]
    Send(String),
    #[error("internal: {0}")]
    Internal(String),
}

/// Trait für Transport-Provider.
/// Implementierungen sind für Thread-Sicherheit selbst verantwortlich.
/// `poll_events` MUSS schnell sein (keine Blockierung / kein Await).
pub trait TransportProvider: Send + Sync {
    /// Eindeutiger Provider-Typ.
    fn kind(&self) -> ProviderKind;

    /// Startet den Provider (Listener aufsetzen, Tasks spawn usw.).
    fn start(&mut self, cfg: Arc<ServerRuntimeConfig>) -> anyhow::Result<()>;

    /// Pull-basiertes Event-Sammeln (füllt Buffer mit 0..n Events).
    fn poll_events(&mut self, out: &mut Vec<ProviderEvent>);

    /// Sendet rohe (already framed) Bytes an ein Connection Ziel.
    fn send(&mut self, id: ConnectionId, bytes: &[u8]) -> Result<(), TransportError>;

    /// Verbindungsabbruch (sanft falls möglich).
    fn disconnect(&mut self, id: ConnectionId, reason: Option<&str>);

    /// Provider stoppen (Listener schließen, Tasks signalisieren).
    fn shutdown(&mut self);

    /// Optional: Downcast Support (z. B. für Tests / Spezialfunktionen).
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Sammlung aktiver Provider.
/// Wird als Bevy-Resource gehalten und beim Startup initialisiert.
// removed duplicate Resource import
#[derive(Resource)]
pub struct ActiveTransports {
    providers: Vec<Box<dyn TransportProvider>>,
    /// Mapping für schnelles Routing (z. B. beim Senden).
    /// Wird bei NewConnection-Events gepflegt.
    conn_index: HashMap<ConnectionId, usize>,
    /// Zwischenspeicher für Events – wird pro Frame extern geleert/verarbeitet.
    staging: Vec<ProviderEvent>,
}

impl ActiveTransports {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            conn_index: HashMap::new(),
            staging: Vec::new(),
        }
    }

    pub fn register_provider(&mut self, provider: Box<dyn TransportProvider>) {
        self.providers.push(provider);
    }

    /// Startet alle Provider – Abbruch bei erstem Fehler.
    pub fn start_all(&mut self, cfg: Arc<ServerRuntimeConfig>) -> anyhow::Result<()> {
        for p in &mut self.providers {
            p.start(cfg.clone())?;
        }
        Ok(())
    }

    /// Pollt alle Provider und sammelt deren Events.
    /// Gibt Slice der gesammelten Events zurück (lebensdauer bis zum nächsten Aufruf).
    pub fn poll(&mut self) -> &[ProviderEvent] {
        self.staging.clear();
        for (idx, p) in self.providers.iter_mut().enumerate() {
            p.poll_events(&mut self.staging);
            // Neue Verbindungen indexieren
            for ev in self.staging.iter() {
                if let ProviderEvent::NewConnection { id, .. } = ev {
                    self.conn_index.insert(*id, idx);
                }
                if let ProviderEvent::Disconnected { id, .. } = ev {
                    self.conn_index.remove(id);
                }
            }
        }
        &self.staging
    }

    /// Sendet rohe (bereits codierte) Daten an Verbindung.
    pub fn send_raw(&mut self, id: ConnectionId, bytes: &[u8]) -> Result<(), TransportError> {
        let prov_idx = self
            .conn_index
            .get(&id)
            .ok_or(TransportError::UnknownConnection)?;
        self.providers[*prov_idx].send(id, bytes)
    }

    /// Verbindungsabbruch (best effort).
    pub fn disconnect(&mut self, id: ConnectionId, reason: Option<&str>) {
        if let Some(idx) = self.conn_index.get(&id).copied() {
            self.providers[idx].disconnect(id, reason);
        }
    }

    /// Stoppt alle Provider (Shutdown).
    pub fn shutdown_all(&mut self) {
        for p in &mut self.providers {
            p.shutdown();
        }
        self.providers.clear();
        self.conn_index.clear();
        self.staging.clear();
    }

    /// Zugriff für Spezialfälle (z. B. Downcast auf konkreten Provider in Tests).
    pub fn providers_mut(&mut self) -> &mut [Box<dyn TransportProvider>] {
        &mut self.providers
    }
}

impl Default for ActiveTransports {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ActiveTransports {
    fn drop(&mut self) {
        // Fail-safe: sicherstellen, dass Ressourcen freigegeben werden.
        self.shutdown_all();
    }
}

/// Thread-sicherer Wrapper (falls du später außerhalb Bevy Threads brauchst).
/// Aktuell optional; kann entfernt werden falls nicht nötig.
#[derive(Clone, Default)]
pub struct SharedTransports(pub Arc<Mutex<ActiveTransports>>);

impl SharedTransports {
    pub fn new(inner: ActiveTransports) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }
}

/// Helper für Logging der ProviderEvent Sammlung (Debugging).
pub fn log_provider_events(events: &[ProviderEvent]) {
    for ev in events {
        match ev {
            ProviderEvent::NewConnection { id, remote, via } => {
                bevy::log::info!(
                    target: "server::net::transport",
                    "Neue Verbindung: {:?} via {:?} remote={}",
                    id,
                    via,
                    remote
                );
            }
            ProviderEvent::Disconnected { id, reason, via } => {
                bevy::log::info!(
                    target: "server::net::transport",
                    "Verbindung geschlossen: {:?} via {:?} reason={:?}",
                    id,
                    via,
                    reason
                );
            }
            ProviderEvent::RawInbound { id, bytes, via } => {
                bevy::log::trace!(
                    target: "server::net::transport",
                    "Raw Frame {} bytes von {:?} via {:?}",
                    bytes.len(),
                    id,
                    via
                );
            }
        }
    }
}

/// System-Skelett (wird später ins Plugin eingebunden):
/// 1. Poll Provider
/// 2. (Optional) Logging
/// 3. Übergabe an Handshake-/Session-Layer (Folgeschritt)
///
/// (Der echte Einbau passiert in lib.rs sobald die restlichen Strukturen stehen.)
#[cfg(feature = "bevy")]
pub fn poll_transport_system(mut transports: bevy::prelude::ResMut<ActiveTransports>) {
    let events = transports.poll();
    log_provider_events(events);
    // TODO: Übergabe an Handshake Manager
}
