//! QUIC Endpoint Skeleton
//!
//! Erweiterung: Accept-Loop Task Spawner hinzugefügt.
//! - Ein neues System `accept_loop_system` pollt non-blocking das `incoming` Stream Objekt,
//!   indem es schrittweise (pro Frame) genau eine Connection akzeptiert (Cooperative Scheduling).
//! - Für jede angenommene Verbindung wird ein Tokio-Task gespawnt (`handle_new_connection_task`),
//!   der später den Handshake State Machine Ablauf ausführen wird.
//! - Aktuell nur Platzhalter-Logging; tatsächliche Handshake-Implementierung folgt in separaten M1 Tasks.
//!
//! Hinweis: Das polling geschieht absichtlich nicht in einer endlosen Schleife innerhalb des Systems,
//! sondern verarbeitet höchstens eine (optionale) Verbindung pro Bevy-Update, um die Main-Thread
//! Frame-Time nicht zu blockieren. Später kann man ein Budget (z. B. N Verbindungen pro Tick) hinzufügen.
//!
//! Ziel:
//! - Zentrale Kapselung der QUIC Listener / Endpoint Logik.
//! - Einheitlicher Ort für: TLS Laden, Quinn `Endpoint` Erzeugung, TransportConfig,
//!   Accept-Loop (Handshake Tasks), Shutdown / Graceful Close.
//!
//! Aktueller Status (Skeleton):
//! - Stellt nur Strukturen, Traits und geplante Funktionen bereit.
//! - Noch keine echte Netzwerk-Initialisierung (folgt in M1 Schritten).
//!
//! Geplante Schritte (chronologisch):
//! 1. TLS Loader implementieren (Zertifikat + Key via Pfade aus `ServerRuntimeConfig`).
//! 2. `build_server_config(rustls::ServerConfig)` + TransportConfig (Idle Timeout, Max Streams).
//! 3. Quinn `Endpoint` binden (`quinn::Endpoint::server()`).
//! 4. Accept Loop (spawn Task / Bevy-System) → eingehende Verbindungen an Handshake State Machine übergeben.
//! 5. Handshake State Machine (Version, Token, SessionId).
//! 6. Session Registry Integration.
//! 7. Metrics + Logs + Error Mapping.
//! 8. Graceful Shutdown (Resource Drop oder Signal).
//!
//! Design Notes:
//! - Endpoint wird als Resource (`QuicEndpointHandle`) in Bevy hinterlegt.
//! - Für testbare Architektur: Bau-Funktion entkoppeln (reiner Funktionsaufruf erzeugt (Endpoint, Incoming)).
//! - Handshake akzeptiert pro Verbindung initial ein Bidirectional-Stream (oder Uni? – Entscheidung folgt).
//! - TLS Dev Self-Signed wird über Feature `debug` optional generiert, falls Dateien fehlen.
//!
//! Sicherheit / Zukunft:
//! - Später: OCSP / Zertifikatsrotation via Control-Plane Hooks.
//! - Rate Limits (SYN Flood Gegenmaßnahmen?) – wahrscheinlich zu früh, erst später nötig.
//!
//! Fehlerbehandlung / Logging Konventionen:
//! - target = "server::net::quic" (initiale Setup Logs)
//! - target = "server::net::handshake" (Handshake Ebene)
//! - target = "server::net::session" (Session Aktionen)
//!
//! Dieses File ist absichtlich kompakt gehalten – Wachstum über Unter-Module wenn nötig:
//! quic/
//!   endpoint.rs
//!   tls.rs (später)
//!   transport.rs (optional)
//!   accept.rs (optional)
//!
//! Hinweis: Alle TODOs sind bewusst nummeriert um sie leichter in Tasks zu überführen.
use super::tls::load_or_generate_tls;
use crate::ServerRuntimeConfig;
use bevy::prelude::*;
use std::{net::SocketAddr, sync::Arc, time::Duration};

/// Öffentliche Resource, welche den aktiven QUIC Listener repräsentiert.
///
/// Später enthält:
/// - `endpoint: quinn::Endpoint`
/// - `server_config: Arc<quinn::ServerConfig>`
/// - `bind_addr: SocketAddr`
/// - evtl. `shutdown_tx` für Graceful Stop
#[derive(Resource, Debug)]
pub struct QuicEndpointHandle {
    pub bind_addr: SocketAddr,
    pub endpoint: quinn::Endpoint,
    // Incoming handle entfernt (quinn 0.11 liefert kein persistierbares Incoming-Stream Objekt mehr)
    pub incoming: (),
}

/// Fehler für Aufbau / Start des QUIC Endpoints.
#[derive(thiserror::Error, Debug)]
pub enum QuicEndpointError {
    #[error("invalid bind address: {0}")]
    InvalidBindAddr(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("tls: {0}")]
    Tls(String),
    #[error("quinn: {0}")]
    Quinn(String),
}

/// Transport-spezifische Parameter (könnten später aus Settings kommen).
#[derive(Debug, Clone)]
pub struct QuicTransportParams {
    pub idle_timeout: Duration,
    pub max_concurrent_bidi_streams: u32,
    pub keep_alive_interval: Option<Duration>,
}

impl Default for QuicTransportParams {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(30),
            max_concurrent_bidi_streams: 64,
            keep_alive_interval: Some(Duration::from_secs(5)),
        }
    }
}

/// Öffentlicher Einstiegspunkt für (zukünftige) Initialisierung.
/// Derzeit nur ein Skeleton das die Bind-Adresse validiert.
pub fn build_quic_endpoint(
    cfg: &ServerRuntimeConfig,
    params: QuicTransportParams,
) -> Result<QuicEndpointHandle, QuicEndpointError> {
    // 1) TLS laden / ggf. generieren
    let tls_config = load_or_generate_tls(cfg)?;
    // 2) Quinn ServerConfig + TransportConfig aufbauen
    //    Quinn 0.11 erwartet ein Objekt, das `quinn::crypto::ServerConfig` implementiert.
    //    Mit aktiviertem `rustls` Feature nutzen wir den Adapter aus `quinn::crypto::rustls`.
    // Build QUIC crypto config via TryFrom (quinn 0.11 expects a QUIC-compatible rustls config)
    let mut server_config = {
        use std::convert::TryFrom;
        let base = Arc::new(tls_config);
        let crypto =
            quinn::crypto::rustls::QuicServerConfig::try_from(base.clone()).map_err(|_| {
                QuicEndpointError::Tls(
                    "invalid TLS server config for QUIC (missing TLS1.3 or cipher suites)".into(),
                )
            })?;
        quinn::ServerConfig::with_crypto(Arc::new(crypto))
    };

    // Transport-Konfiguration
    let mut transport = quinn::TransportConfig::default();
    // Idle Timeout (Duration -> VarInt Millisekunden konvertieren)
    let idle_ms = params
        .idle_timeout
        .as_millis()
        .clamp(1, u128::from(u32::MAX)) as u32;
    transport.max_idle_timeout(Some(quinn::IdleTimeout::from(quinn::VarInt::from_u32(
        idle_ms,
    ))));
    // Max BiDi Streams
    transport
        .max_concurrent_bidi_streams(quinn::VarInt::from_u32(params.max_concurrent_bidi_streams));
    // Keep-Alive
    if let Some(iv) = params.keep_alive_interval {
        transport.keep_alive_interval(Some(iv));
    }

    server_config.transport_config(Arc::new(transport));

    // 3) Endpoint binden
    let addr: SocketAddr = format!("{}:{}", cfg.bind_addr, cfg.port)
        .parse()
        .map_err(|_| {
            QuicEndpointError::InvalidBindAddr(format!("{}:{}", cfg.bind_addr, cfg.port))
        })?;

    // Quinn 0.11 gibt hier nur das Endpoint-Objekt zurück; `Incoming` erhalten wir via `accept()`.
    let endpoint = quinn::Endpoint::server(server_config, addr)
        .map_err(|e| QuicEndpointError::Quinn(format!("endpoint server: {e}")))?;
    // quinn 0.11: we no longer store an Incoming/Accept handle; accept loop will be implemented elsewhere
    let incoming = ();

    bevy::log::info!(
        target: "server::net::quic",
        "QUIC endpoint gebunden {}:{} (ALPNs={:?}, bidi_streams={}, idle_timeout={:?})",
        cfg.bind_addr,
        cfg.port,
        cfg.alpn,
        params.max_concurrent_bidi_streams,
        params.idle_timeout
    );

    Ok(QuicEndpointHandle {
        bind_addr: addr,
        endpoint,
        // incoming removed in quinn 0.11 API (placeholder retained as unit)
        incoming,
    })
}

/// System: Baut (wenn nicht vorhanden) den QUIC Endpoint auf Basis der aktuellen Runtime-Config.
/// Aktuell nur Platzhalter, später ersetzt durch echte Quinn Initialisierung.
pub fn ensure_quic_endpoint_system(
    _commands: Commands,
    _existing: Option<Res<QuicEndpointHandle>>,
) {
    // QUIC endpoint skeleton (disabled for quinn 0.11 migration) – no-op.
}

/// System: (Zukünftig) Accept-Loop triggern oder Task am Laufen halten.
/// Derzeit nur Debug-Log falls Endpoint existiert.
pub fn quic_tick_system(endpoint: Option<Res<QuicEndpointHandle>>) {
    if let Some(ep) = endpoint {
        bevy::log::trace!(
            target: "server::net::quic",
            "QUIC tick (stub) – endpoint at {}",
            ep.bind_addr
        );
    }
}

/// Accept loop stub (temporarily disabled).
/// The previous skeleton referenced a removed incoming stream type. A proper
/// accept loop will be reintroduced once the higher-level transport layer
/// provides a unified spawning mechanism for connection tasks.
pub fn accept_loop_system(_endpoint: Option<Res<QuicEndpointHandle>>) {
    // Intentionally left blank.
}

/// Placeholder Task für neue eingehende QUIC-Verbindung.
/// Später:
/// - Perform Handshake (Version, Auth-Token)
/// - Frame-Layer initialisieren
/// - Session registrieren
async fn handle_new_connection_task(connecting: quinn::Connecting) -> anyhow::Result<()> {
    match connecting.await {
        Ok(connection) => {
            let remote = connection.remote_address();
            bevy::log::info!(
                target:"server::net::quic",
                "Neue Verbindung akzeptiert: {}", remote
            );

            // TODO(M1-HS): Handshake State Machine starten
            // z.B. ersten BiDi Stream öffnen / warten und Handshake Frames austauschen.

            // Platzhalter: kurz warten und schließen (Demonstration)
            // In echter Implementierung: Session persistieren, read loop starten, etc.
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("connecting failed: {e}")),
    }
}

/// Graceful Shutdown (Drop Hook / optionales System).
/// In Zukunft: Endpoint schließen, offene Sessions benachrichtigen.
impl Drop for QuicEndpointHandle {
    fn drop(&mut self) {
        bevy::log::info!(
            target: "server::net::quic",
            "QUIC Endpoint Handle dropped ({}). Graceful shutdown stub.",
            self.bind_addr
        );
    }
}

/// Trait für zukünftige TLS Bereitstellung (evtl. ausgelagert in eigenes Modul).
pub trait TlsProvider: Send + Sync {
    fn load(
        &self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<Arc<rustls::ServerConfig>, QuicEndpointError>;
}

/// Dev/Prod TLS Loader Platzhalter.
pub struct DefaultTlsProvider;

impl TlsProvider for DefaultTlsProvider {
    fn load(
        &self,
        _cert_path: &str,
        _key_path: &str,
    ) -> Result<Arc<rustls::ServerConfig>, QuicEndpointError> {
        // TODO(M1-TLS-01): Dateien einlesen, Zert+Key parsen
        // TODO(M1-TLS-02): rustls::ServerConfig erstellen (ALPN setzen)
        // TODO(M1-TLS-03): Dev Self-Signed generieren (Feature `debug`) falls Dateien fehlen
        Err(QuicEndpointError::Tls(
            "TLS Provider not implemented (skeleton)".into(),
        ))
    }
}

// -------------------------------------------------------------------------------------------------
// Tests (reine Strukturtests – keine echte QUIC Bindung hier).
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    fn dummy_cfg() -> ServerRuntimeConfig {
        ServerRuntimeConfig {
            bind_addr: "127.0.0.1".into(),
            port: 47000,
            cert_path: "certs/dev.crt".into(),
            key_path: "certs/dev.key".into(),
            alpn: vec!["aether/0.1".into()],
            metrics_enabled: true,
            tick_rate: 60.0,
            uds_path: "/tmp/aether.sock".into(),
            handshake_timeout_ms: 5000,
            max_sessions: 100,
            max_frame_bytes: 1024,
            max_idle_timeout_ms: 30000,
            keep_alive_interval_ms: 10000,
            max_concurrent_bidi_streams: 100,
            max_concurrent_uni_streams: 50,
            mtu: 1500,
            initial_congestion_window: 1000,
            client_ip_migration: true,
            zero_rtt_resumption: true,
            qos_traffic_prioritization: true,
            nat_traversal: true,
        }
    }

    #[test]
    fn build_endpoint_basic() {
        let cfg = dummy_cfg();
        let ep = build_quic_endpoint(&cfg, QuicTransportParams::default())
            .expect("skeleton endpoint build");
        assert_eq!(ep.bind_addr.port(), 47000);
    }

    #[test]
    fn invalid_bind_addr() {
        let mut cfg = dummy_cfg();
        cfg.bind_addr = "not-an-ip".into();
        let err = build_quic_endpoint(&cfg, QuicTransportParams::default()).unwrap_err();
        match err {
            QuicEndpointError::InvalidBindAddr(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
