/*!
Network Server Crate (Forge of Stories)

Ziel dieses Rewrites:
- Keine eigene redundante Config-Struktur mehr im Server-Crate.
- Direkte Nutzung der bereits in `aether_config` registrierten Settings-Sektionen.
- Aggregation aller relevanten Werte in eine leicht konsumierbare Snapshot-Resource (`ServerRuntimeConfig`).
- Hot-Reload durch Arc-Pointer-Vergleich (keine teuren Deep-Comparisons).
- Saubere Basis für kommende Implementationen (QUIC/TLS Handshake, Sessions, Frames).

Voraussetzungen (zur Laufzeit):
- Die Bevy-App MUSS zuvor `use_aether_server_settings()` (aus `aether_config::bevy`) aufgerufen haben.
  Dadurch werden:
    * SettingsStoreRef
    * SettingsArc<GeneralCfg>, SettingsArc<NetworkCfg>, SettingsArc<SecurityCfg>,
      SettingsArc<MonitoringCfg>, SettingsArc<UdsCfg>
  als Resources bereitgestellt.

Dieses Crate stellt aktuell KEINE echten Netzwerk-Funktionen bereit – dafür folgen Milestone M1 Tasks
(QUIC Listener, Handshake, Session Management usw.). Der Fokus liegt auf sauberer Settings-Integration.

Öffentliche API:
- Plugin: `NetworkServerPlugin`
- Snapshot Resource: `ServerRuntimeConfig` (über `RuntimeConfigSnapshot`)
- Helper: `get_runtime_config(&World) -> Option<Arc<ServerRuntimeConfig>>`
- Prelude: `server::prelude::*`

Weitere geplante Module (noch nicht implementiert):
- handshake (State Machine)
- session (Registry)
- frames / codec (Transport)
- metrics / rate limiting
*/

use aether_config::{GeneralCfg, MonitoringCfg, NetworkCfg, SecurityCfg, UdsCfg};
use bevy::prelude::*;
use settings::SettingsArc;
use std::sync::Arc;

// -------------------------------------------------------------------------------------------------
// Aggregierte Runtime-Config
// -------------------------------------------------------------------------------------------------

/// Aggregiertes, schreibgeschütztes Snapshot der aktuell effektiven Server-Konfiguration.
/// Dieses Objekt wird (per Arc) an subsystems / Netzwerk-Tasks weitergereicht.
///
/// Erweiterbar: Sobald zusätzliche Felder (handshake_timeout_ms, max_frame_bytes, etc.)
/// in den bestehenden Aether-Settings auftauchen, können sie hier ergänzt werden.
///
/// Wichtig: Keine direkten `SettingsArc<T>` Felder hier, sondern entkoppelte, kopierte Werte
/// für konsistente Snapshots (Atomic Swap).
#[derive(Debug, Clone)]
pub struct ServerRuntimeConfig {
    pub bind_addr: String,
    pub port: u16,
    pub cert_path: String,
    pub key_path: String,
    pub alpn: Vec<String>,
    pub metrics_enabled: bool,
    pub tick_rate: f64,
    pub uds_path: String,
    // --- Handshake / Frame Limits ---
    /// Handshake timeout in milliseconds (pending connections are dropped after this).
    pub handshake_timeout_ms: u64,
    /// Maximum allowed serialized frame size (payload, excluding length prefix).
    pub max_frame_bytes: u32,
    /// Upper bound for concurrently established sessions.
    pub max_sessions: u32,
    // --- QUIC Transport Tuning (derived from [network] settings) ---
    /// QUIC idle timeout in milliseconds.
    pub max_idle_timeout_ms: u64,
    /// QUIC keep-alive interval in milliseconds.
    pub keep_alive_interval_ms: u64,
    pub max_concurrent_bidi_streams: u32,
    pub max_concurrent_uni_streams: u32,
    pub mtu: u32,
    pub initial_congestion_window: u32,
    pub client_ip_migration: bool,
    pub zero_rtt_resumption: bool,
    pub qos_traffic_prioritization: bool,
    pub nat_traversal: bool,
}

impl ServerRuntimeConfig {
    fn from_sections(
        net: &NetworkCfg,
        sec: &SecurityCfg,
        mon: &MonitoringCfg,
        general: &GeneralCfg,
        uds: &UdsCfg,
    ) -> Self {
        Self {
            bind_addr: net.ip_address.clone(),
            port: net.udp_port,
            cert_path: sec.cert_path.clone(),
            key_path: sec.key_path.clone(),
            alpn: sec.alpn.clone(),
            metrics_enabled: mon.metrics_enabled,
            tick_rate: general.tick_rate,
            uds_path: uds.path.clone(),
            // Security-derived (seconds -> ms)
            handshake_timeout_ms: sec.handshake_timeout.saturating_mul(1000),
            max_frame_bytes: sec.max_frame_bytes,
            max_sessions: sec.max_sessions,
            // QUIC transport (seconds -> ms where applicable)
            max_idle_timeout_ms: net.max_idle_timeout.saturating_mul(1000),
            keep_alive_interval_ms: net.keep_alive_interval.saturating_mul(1000),
            max_concurrent_bidi_streams: net.max_concurrent_bidi_streams,
            max_concurrent_uni_streams: net.max_concurrent_uni_streams,
            mtu: net.mtu,
            initial_congestion_window: net.initial_congestion_window,
            client_ip_migration: net.client_ip_migration,
            zero_rtt_resumption: net.zero_rtt_resumption,
            qos_traffic_prioritization: net.qos_traffic_prioritization,
            nat_traversal: net.nat_traversal,
        }
    }
}

/// Interne Resource, die neben dem aktuellen Snapshot auch die zuletzt gesehenen Arc-Pointer enthält
/// (für Change Detection ohne Deep Compare).
#[derive(Resource, Clone)]
struct RuntimeConfigSnapshot {
    config: Arc<ServerRuntimeConfig>,
    last_net: Arc<NetworkCfg>,
    last_sec: Arc<SecurityCfg>,
    last_mon: Arc<MonitoringCfg>,
    last_gen: Arc<GeneralCfg>,
    last_uds: Arc<UdsCfg>,
}

// -------------------------------------------------------------------------------------------------
// Plugin
// -------------------------------------------------------------------------------------------------

pub struct NetworkServerPlugin;

// QUIC module skeleton (stub – echte Implementierung folgt in M1)
pub mod quic {
    /*!
    QUIC Module – Forge of Stories Server

    Enthält (derzeit als Skeleton):
    - `endpoint` Modul: Aufbau / Platzhalter für QUIC Endpoint und zukünftige Accept-Loop.
    - Re-Exports der wichtigsten Typen & Funktionen für einfache Nutzung in `server`-Code.

    Geplante Erweiterungen (M1 Schritte):
    1. TLS Laden (Zertifikat + Key aus Settings / RuntimeConfig).
    2. Quinn `ServerConfig` + `TransportConfig`.
    3. Endpoint-Bind + Accept Loop (eingehende Verbindungen → Handshake State Machine).
    4. Handshake Frames (Version, Token) + SessionId Vergabe.
    5. Session Registry Integration.
    6. Ping/Pong + Frame Codec Integration.
    7. Limits & Rate-Limit Skeleton.
    8. Metrics & Logging (Counter).
    9. Graceful Shutdown (Drop / Signal).

    Logging Targets (Vorschlag):
    - server::net::quic        – Setup & Low-Level Transport
    - server::net::handshake   – Handshake State Machine
    - server::net::session     – Session Insert/Remove
    - server::net::frames      – Frame Encode/Decode Events
    - server::net::metrics     – Periodische Dumps

    Dieses Modul bleibt bewusst schmal; spezialisierte Teile wandern später in Unter-Module (z. B. tls.rs, accept.rs).
    */

    pub(crate) mod endpoint;
    pub(crate) mod tls;
    pub use endpoint::{
        DefaultTlsProvider, QuicEndpointError, QuicEndpointHandle, QuicTransportParams,
        TlsProvider, build_quic_endpoint, ensure_quic_endpoint_system, quic_tick_system,
    };
}

pub mod protocol;
pub mod session;
pub mod transport;

use crate::protocol::handshake::{
    NetSessionClosed, NetSessionEstablished, default_pending_handshakes,
    handshake_process_transport_events_system, handshake_timeout_system,
};
use crate::protocol::metrics::{
    metrics_dump_system, metrics_session_events_system, setup_metrics_resources,
};
use crate::session::SessionRegistry;
// Importiere Transport-Typen explizit über crate:: Pfad um Namensauflösung eindeutig zu machen.
//
// Lokaler Fallback-Event-Buffer (da TransportEventBuffer im transport-Modul feature-gated ist)
#[derive(Resource, Default)]
struct TransportEventBuffer {
    events: Vec<transport::ProviderEvent>,
}
use crate::protocol::codec::{FrameCodec, FrameDecoder};
use crate::protocol::frames::{Frame, TransportFrame};
use crate::protocol::metrics::NetMetrics;
use crate::transport::{
    ActiveTransports, ConnectionId, quic_provider::boxed_quic_provider,
    steam_provider::boxed_steam_provider,
};
use std::collections::HashMap;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        // Minimaler Guard: Wenn Settings-Ressourcen fehlen, loggen wir einen Fehler.
        {
            let world = app.world();
            let missing = !(world.contains_resource::<SettingsArc<NetworkCfg>>()
                && world.contains_resource::<SettingsArc<SecurityCfg>>()
                && world.contains_resource::<SettingsArc<MonitoringCfg>>()
                && world.contains_resource::<SettingsArc<GeneralCfg>>()
                && world.contains_resource::<SettingsArc<UdsCfg>>());
            if missing {
                bevy::log::error!(
                    target: "server::net::config",
                    "NetworkServerPlugin: Erwartete SettingsArc<...>-Ressourcen fehlen. \
                     Bitte zuvor `use_aether_server_settings()` aufrufen."
                );
            }
        }

        // 1) RuntimeConfig Snapshot bauen
        app.add_systems(Startup, build_runtime_config_once);
        // 2) Transport Layer initialisieren (nachdem Config existiert)
        app.add_systems(Startup, init_transports_system);
        // 2b) Protokoll-Ressourcen (Sessions, Pending Handshakes, Metrics)
        app.add_systems(
            Startup,
            init_protocol_resources_system.after(init_transports_system),
        );

        // Events registrieren (Sessions)
        app.add_event::<NetSessionEstablished>();
        app.add_event::<NetSessionClosed>();

        // 3) Laufende Updates: Config-Änderungen + Transport / Handshake / Metrics
        app.add_systems(Update, update_runtime_config_if_changed);
        app.add_systems(Update, poll_transports_system);
        app.add_systems(
            Update,
            handshake_process_transport_events_system.after(poll_transports_system),
        );
        app.add_systems(
            Update,
            session_disconnect_and_frame_router_system
                .after(handshake_process_transport_events_system),
        );
        app.add_systems(Update, handshake_timeout_system);
        app.add_systems(Update, metrics_session_events_system);
        app.add_systems(Update, metrics_dump_system);
    }
}

// -------------------------------------------------------------------------------------------------
// Systems
// -------------------------------------------------------------------------------------------------

fn build_runtime_config_once(
    net: Option<Res<SettingsArc<NetworkCfg>>>,
    sec: Option<Res<SettingsArc<SecurityCfg>>>,
    mon: Option<Res<SettingsArc<MonitoringCfg>>>,
    general: Option<Res<SettingsArc<GeneralCfg>>>,
    uds: Option<Res<SettingsArc<UdsCfg>>>,
    mut commands: Commands,
) {
    let (Some(net), Some(sec), Some(mon), Some(general), Some(uds)) = (net, sec, mon, general, uds)
    else {
        return;
    };

    let cfg = ServerRuntimeConfig::from_sections(&net, &sec, &mon, &general, &uds);
    commands.insert_resource(RuntimeConfigSnapshot {
        config: Arc::new(cfg),
        last_net: net.0.clone(),
        last_sec: sec.0.clone(),
        last_mon: mon.0.clone(),
        last_gen: general.0.clone(),
        last_uds: uds.0.clone(),
    });
    bevy::log::info!(
        target: "server::net::config",
        "RuntimeConfig initialisiert ({}:{})",
        net.ip_address,
        net.udp_port
    );
}

/// Initialisiert die Transport-Provider je nach Modus.
/// Aktuell: Immer QUIC Provider (bis weitere Modi implementiert sind).
fn init_transports_system(runtime: Option<Res<RuntimeConfigSnapshot>>, mut commands: Commands) {
    if let Some(rt) = runtime {
        let mut at = ActiveTransports::new();
        // Initialisiere globalen Event-Buffer (wird pro Frame zurückgesetzt)
        commands.insert_resource(TransportEventBuffer::default());

        // Mode Dispatch (provisorisch):
        // Reihenfolge der Provider je nach Modus:
        // dedicated -> QUIC
        // relay     -> Steam (Stub)
        // hybrid    -> Steam + QUIC
        // local     -> (vorerst nur QUIC; Local/InProc folgt später)
        let mode = std::env::var("FOS_NETWORK_MODE")
            .ok()
            .unwrap_or_else(|| "dedicated".to_string())
            .to_lowercase();

        let mut registered: Vec<&'static str> = Vec::new();
        match mode.as_str() {
            "relay" => {
                at.register_provider(boxed_steam_provider());
                registered.push("Steam");
            }
            "hybrid" => {
                at.register_provider(boxed_steam_provider());
                at.register_provider(boxed_quic_provider());
                registered.push("Steam");
                registered.push("QUIC");
            }
            "local" => {
                // Platzhalter: QUIC weiterhin als Transport bis LocalProvider existiert.
                at.register_provider(boxed_quic_provider());
                registered.push("QUIC");
            }
            "dedicated" | _ => {
                at.register_provider(boxed_quic_provider());
                registered.push("QUIC");
            }
        }

        // Start alle Provider
        let cfg_arc = rt.config.clone();
        if let Err(e) = at.start_all(cfg_arc) {
            bevy::log::error!(
                target: "server::net::transport",
                "Transport Start fehlgeschlagen (mode={mode}): {e}"
            );
            return;
        }

        bevy::log::info!(
            target: "server::net::transport",
            "[mode={}] registered transports: {}",
            mode,
            registered.join(", ")
        );

        commands.insert_resource(at);
    } else {
        bevy::log::warn!(
            target: "server::net::transport",
            "init_transports_system: RuntimeConfigSnapshot fehlt – übersprungen"
        );
    }
}

/// Pollt alle aktiven Transports und leitet Events weiter.
/// Aktuell: Nur Logging. Später: Übergabe an Handshake/Session Layer.
fn poll_transports_system(
    mut transports: ResMut<ActiveTransports>,
    mut buffer: ResMut<TransportEventBuffer>,
) {
    let events = transports.poll();
    if events.is_empty() {
        return;
    }
    // Rekonstruiere Events (ProviderEvent ist nicht Clone)
    for ev in events {
        match ev {
            transport::ProviderEvent::NewConnection { id, remote, via } => {
                buffer.events.push(transport::ProviderEvent::NewConnection {
                    id: *id,
                    remote: remote.clone(),
                    via: *via,
                });
            }
            transport::ProviderEvent::Disconnected { id, reason, via } => {
                buffer.events.push(transport::ProviderEvent::Disconnected {
                    id: *id,
                    reason: reason.clone(),
                    via: *via,
                });
            }
            transport::ProviderEvent::RawInbound { id, bytes, via } => {
                buffer.events.push(transport::ProviderEvent::RawInbound {
                    id: *id,
                    bytes: bytes.clone(),
                    via: *via,
                });
            }
        }
    }
    transport::log_provider_events(&buffer.events);
}

/// Einfache Ping/Pong Verarbeitung für bereits etablierte Sessions.
/// Hinweis: Dies pollt erneut. In einer späteren Refactor-Runde sollte
/// das doppelte Polling durch eine zentrale Dispatch-Schicht ersetzt werden.
#[derive(Resource, Default)]
struct SessionFrameDecoders {
    map: HashMap<ConnectionId, FrameDecoder>,
}

fn session_disconnect_and_frame_router_system(
    mut transports: ResMut<ActiveTransports>,
    runtime: Option<Res<RuntimeConfigSnapshot>>,
    mut sessions: ResMut<SessionRegistry>,
    mut decoders: ResMut<SessionFrameDecoders>,
    mut ev_closed: EventWriter<NetSessionClosed>,
    mut metrics: ResMut<NetMetrics>,
    mut buffer: ResMut<TransportEventBuffer>,
) {
    let Some(rt) = runtime else { return };

    // Events aus zentralem Buffer entnehmen (Poll bereits in poll_transports_system erfolgt).
    let events = std::mem::take(&mut buffer.events);
    if events.is_empty() {
        return;
    }
    // Verarbeite Events: Disconnect zuerst, dann Inbound
    for ev in events {
        match ev {
            transport::ProviderEvent::Disconnected { id, .. } => {
                if let Some(session_id) = sessions.by_conn.get(&id).copied() {
                    if let Some(meta) = sessions.remove(session_id) {
                        decoders.map.remove(&meta.connection_id);
                        ev_closed.write(NetSessionClosed(session_id));
                        metrics.active_sessions = metrics.active_sessions.saturating_sub(1);
                        bevy::log::info!(
                            target:"server::net::session",
                            "Session geschlossen id={} conn={:?}",
                            session_id,
                            meta.connection_id
                        );
                    }
                }
            }
            transport::ProviderEvent::NewConnection { .. } => {
                // Decoder erst nach erfolgreichem Handshake anlegen (im Handshake-System)
            }
            transport::ProviderEvent::RawInbound { id, bytes, .. } => {
                // Nur etablierte Sessions
                let Some(session_id) = sessions.by_conn.get(&id).copied() else {
                    continue;
                };
                // Decoder holen/erstellen
                let dec = decoders
                    .map
                    .entry(id)
                    .or_insert_with(|| FrameDecoder::new(rt.config.max_frame_bytes));
                dec.push_bytes(&bytes);
                // Frames verarbeiten
                loop {
                    match dec.next_frame() {
                        Ok(Some(frame)) => {
                            match frame {
                                Frame::Transport(TransportFrame::Ping(v)) => {
                                    // Antwort
                                    let reply = Frame::Transport(TransportFrame::Pong(v));
                                    let mut out = Vec::new();
                                    let codec = FrameCodec::new(rt.config.max_frame_bytes);
                                    if codec.encode(&reply, &mut out).is_ok() {
                                        if let Err(e) = transports.send_raw(id, &out) {
                                            bevy::log::debug!(
                                                target:"server::net::transport",
                                                "Ping->Pong send fehlgeschlagen conn={:?}: {e}",
                                                id
                                            );
                                        }
                                    }
                                }
                                Frame::Transport(TransportFrame::Pong(_)) => {
                                    // Liveness Antwort – optional: last_frame_at aktualisieren
                                    sessions.touch(session_id);
                                }
                                Frame::Handshake(_) => {
                                    // Handshake Frames hier ignorieren (sollten nicht mehr auftauchen)
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            bevy::log::debug!(
                                target:"server::net::frames",
                                "Decode Fehler session conn={:?}: {e}",
                                id
                            );
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn update_runtime_config_if_changed(
    net: Option<Res<SettingsArc<NetworkCfg>>>,
    sec: Option<Res<SettingsArc<SecurityCfg>>>,
    mon: Option<Res<SettingsArc<MonitoringCfg>>>,
    general: Option<Res<SettingsArc<GeneralCfg>>>,
    uds: Option<Res<SettingsArc<UdsCfg>>>,
    snapshot: Option<ResMut<RuntimeConfigSnapshot>>,
) {
    let (Some(net), Some(sec), Some(mon), Some(general), Some(uds), Some(mut snap)) =
        (net, sec, mon, general, uds, snapshot)
    else {
        return;
    };

    let changed = !Arc::ptr_eq(&snap.last_net, &net.0)
        || !Arc::ptr_eq(&snap.last_sec, &sec.0)
        || !Arc::ptr_eq(&snap.last_mon, &mon.0)
        || !Arc::ptr_eq(&snap.last_gen, &general.0)
        || !Arc::ptr_eq(&snap.last_uds, &uds.0);

    if changed {
        let new_cfg = ServerRuntimeConfig::from_sections(&net, &sec, &mon, &general, &uds);
        snap.config = Arc::new(new_cfg);
        snap.last_net = net.0.clone();
        snap.last_sec = sec.0.clone();
        snap.last_mon = mon.0.clone();
        snap.last_gen = general.0.clone();
        snap.last_uds = uds.0.clone();
        bevy::log::info!(
            target: "server::net::config",
            "RuntimeConfig aktualisiert"
        );
    }
}

// -------------------------------------------------------------------------------------------------
// Additional Init (Protocol Resources)
// -------------------------------------------------------------------------------------------------

fn init_protocol_resources_system(
    mut commands: Commands,
    runtime: Option<Res<RuntimeConfigSnapshot>>,
) {
    // Werte aus RuntimeConfigSnapshot bevorzugen; Fallback auf Defaults falls Snapshot fehlt.
    let (handshake_timeout, max_frame_bytes, max_sessions) = if let Some(rt) = runtime {
        let cfg = rt.config.as_ref();
        (
            std::time::Duration::from_millis(cfg.handshake_timeout_ms),
            cfg.max_frame_bytes,
            cfg.max_sessions,
        )
    } else {
        bevy::log::warn!(
            target:"server::net::handshake",
            "RuntimeConfigSnapshot fehlt beim Init – verwende Defaults"
        );
        (std::time::Duration::from_secs(5), 64 * 1024, 100_000)
    };

    commands.insert_resource(SessionRegistry::new(max_sessions));
    commands.insert_resource(default_pending_handshakes(
        handshake_timeout,
        max_frame_bytes,
        max_sessions,
    ));
    // Decoder-Registry für etablierte Sessions
    commands.insert_resource(SessionFrameDecoders::default());
    setup_metrics_resources(&mut commands, std::time::Duration::from_secs(10));
    bevy::log::info!(
        target: "server::net::handshake",
        "Protocol resources initialisiert (timeout={:?}, max_sessions={}, frame_max={})",
        handshake_timeout,
        max_sessions,
        max_frame_bytes
    );
}

// -------------------------------------------------------------------------------------------------
// Public Helpers
// -------------------------------------------------------------------------------------------------

/// Liefert eine Arc-Kopie der aktuellen RuntimeConfig, falls vorhanden.
pub fn get_runtime_config(world: &World) -> Option<Arc<ServerRuntimeConfig>> {
    world
        .get_resource::<RuntimeConfigSnapshot>()
        .map(|s| s.config.clone())
}

// -------------------------------------------------------------------------------------------------
// Future Stubs (Vorbereitungs-Namespace)
// -------------------------------------------------------------------------------------------------

/// Geplanter Namespace für kommende Implementierungen (Handshake, Transport, Sessions).
pub mod future {
    //! Platzhalter für zukünftige Netzwerk-Funktionen (M1 und darüber hinaus).
    //!
    //! Geplante Module:
    //! - handshake
    //! - session
    //! - frames
    //! - codec
    //! - metrics / rate
    //!
    //! Diese werden aktiviert, sobald die QUIC/TLS Ebene implementiert wird.
    //!
    //! Aktuell bewusst leer.
    pub struct Placeholder;
}

// -------------------------------------------------------------------------------------------------
// Prelude
// -------------------------------------------------------------------------------------------------

pub mod prelude {
    //! Bequeme Re-Exports für Anwender dieses Crates.
    pub use super::{NetworkServerPlugin, ServerRuntimeConfig, get_runtime_config};
}

// -------------------------------------------------------------------------------------------------
// Tests (Basis)
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use aether_config::bevy::AppAetherSettingsExt;

    #[test]
    fn plugin_initializes_snapshot_if_settings_present() {
        let mut app = App::new();
        // Simuliere vollständige Settings-Initialisierung
        app = app.use_aether_server_settings(None);
        app.add_plugins(MinimalPlugins);
        app.add_plugins(NetworkServerPlugin);
        app.update(); // Startup
        assert!(
            app.world()
                .get_resource::<RuntimeConfigSnapshot>()
                .is_some(),
            "RuntimeConfigSnapshot sollte nach Startup vorhanden sein"
        );
    }
}
