/*!
Handshake State Machine (Forge of Stories – Network Server)

Milestone M1 Scope:
- Track connections that are in "pending handshake" state.
- Accept exactly one initial frame: Frame::Handshake(ClientHello { .. })
- Validate version (SUPPORTED_VERSIONS) + (placeholder) token.
- Upon success: allocate SessionId (via SessionRegistry), send ServerHello, emit NetSessionEstablished.
- Upon failure: send HandshakeError + disconnect.
- Timeout stale pending handshakes.

Out of Scope (future):
- Real token / auth validation (M1-HS-AUTH).
- Rate limiting / flood protection (M1-HS-RATE).
- Multi-packet client hello or fragmentation.
- Session resumption / zero-RTT semantics.
- Metrics counters (placeholder TODO tags).

Integrations:
- Relies on `ActiveTransports` (transport/mod.rs).
- Uses `Frame`, `HandshakeFrame` (protocol/frames.rs) and FrameCodec (protocol/codec.rs).
- Needs `SessionRegistry` (session.rs) to exist with:
    - fn allocate_session_id(&mut self) -> SessionId
    - fn insert(&mut self, id: SessionId, conn: ConnectionId, provider: ProviderKind)

Logging Targets (suggested):
- server::net::handshake  (state transitions / errors)
- server::net::frames     (frame encode/decode trace – optional)

TODO Tags (searchable):
- TODO(M1-HS-VERSION)
- TODO(M1-HS-AUTH)
- TODO(M1-HS-RATELIMIT)
- TODO(M1-HS-METRICS)

(C) Forge of Stories
*/

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use bevy::prelude::*;

use super::codec::FrameCodec;
use super::frames::{Frame, HandshakeErrorCode, HandshakeFrame};
use super::metrics::NetMetrics;
use crate::session::{SessionId, SessionRegistry};
use crate::transport::{
    ActiveTransports, ConnectionId, ProviderEvent, ProviderKind, TransportEventBuffer,
};

/// Supported protocol versions for initial negotiation.
/// Extend carefully; consider highest-first ordering if implementing preference logic later.
pub const SUPPORTED_VERSIONS: &[u16] = &[1];

/// Internal pending handshake record.
#[derive(Debug)]
pub struct PendingHandshake {
    pub started_at: Instant,
    pub buffer: Vec<u8>,
    pub provider: ProviderKind,
}

impl PendingHandshake {
    pub fn new(provider: ProviderKind) -> Self {
        Self {
            started_at: Instant::now(),
            buffer: Vec::new(),
            provider,
        }
    }
}

/// Resource tracking all pending handshakes.
#[derive(Resource, Debug)]
pub struct PendingHandshakes {
    pub map: HashMap<ConnectionId, PendingHandshake>,
    pub timeout: Duration,
    pub max_frame_bytes: u32,
    pub max_sessions: u32,
}

impl PendingHandshakes {
    pub fn new(timeout: Duration, max_frame_bytes: u32, max_sessions: u32) -> Self {
        Self {
            map: HashMap::new(),
            timeout,
            max_frame_bytes,
            max_sessions,
        }
    }
}

/// Event emitted when the server establishes a session (after successful handshake).
#[derive(Event, Debug, Clone, Copy)]
pub struct NetSessionEstablished(pub SessionId);

/// Event emitted when a session terminates (disconnect / error / shutdown).
#[derive(Event, Debug, Clone, Copy)]
pub struct NetSessionClosed(pub SessionId);

/// System: Poll transports, feed events into handshake pipeline.
///
/// Ordering:
/// - Should run AFTER the raw transport poll system (which updates ActiveTransports).
pub fn handshake_process_transport_events_system(
    mut transports: ResMut<ActiveTransports>,
    mut pending: ResMut<PendingHandshakes>,
    mut sessions: ResMut<SessionRegistry>,
    mut ev_established: EventWriter<NetSessionEstablished>,
    mut metrics: ResMut<NetMetrics>,
    mut buffer: ResMut<TransportEventBuffer>,
) {
    // Events aus zentralem TransportEventBuffer entnehmen (Polling passiert separat im poll_transports_system).
    let events = std::mem::take(&mut buffer.events);
    if events.is_empty() {
        return;
    }

    for ev in events {
        match ev {
            ProviderEvent::NewConnection { id, via, remote } => {
                pending.map.insert(id, PendingHandshake::new(via));
                bevy::log::info!(
                    target: "server::net::handshake",
                    "Handshake started conn={:?} via={:?} remote={}",
                    id, via, remote
                );
            }
            ProviderEvent::Disconnected { id, .. } => {
                if pending.map.remove(&id).is_some() {
                    bevy::log::debug!(
                        target: "server::net::handshake",
                        "Pending handshake aborted by disconnect conn={:?}",
                        id
                    );
                }
            }
            ProviderEvent::RawInbound { id, bytes, .. } => {
                if let Some(ph) = pending.map.get_mut(&id) {
                    ph.buffer.extend_from_slice(&bytes);
                } else {
                    bevy::log::trace!(
                        target: "server::net::handshake",
                        "RawInbound for non-pending conn={:?} ignored (session or unknown)",
                        id
                    );
                    continue;
                }
                let max_frame_bytes = pending.max_frame_bytes;
                let mut decoded_frames = Vec::new();
                loop {
                    let decode_result = {
                        if let Some(ph) = pending.map.get_mut(&id) {
                            FrameCodec::try_decode(&mut ph.buffer, max_frame_bytes)
                        } else {
                            break;
                        }
                    };
                    match decode_result {
                        Ok(Some(frame)) => decoded_frames.push(frame),
                        Ok(None) => break,
                        Err(e) => {
                            bevy::log::warn!(
                                target: "server::net::handshake",
                                "Decode failure conn={:?}: {e}",
                                id
                            );
                            send_handshake_error(
                                id,
                                HandshakeErrorCode::Malformed,
                                "decode",
                                &mut transports,
                                max_frame_bytes,
                            );
                            metrics.handshake_fail_malformed =
                                metrics.handshake_fail_malformed.saturating_add(1);
                            transports.disconnect(id, Some("malformed"));
                            pending.map.remove(&id);
                            break;
                        }
                    }
                }
                for frame in decoded_frames {
                    if !pending.map.contains_key(&id) {
                        break;
                    }
                    if let Err(e) = process_handshake_frame(
                        id,
                        frame,
                        &mut transports,
                        &mut pending,
                        &mut sessions,
                        &mut metrics,
                        &mut ev_established,
                    ) {
                        bevy::log::warn!(
                            target: "server::net::handshake",
                            "Handshake error conn={:?}: {e}",
                            id
                        );
                        transports.disconnect(id, Some("handshake error"));
                        pending.map.remove(&id);
                        break;
                    }
                }
            }
        }
    }
}

/// Process a single decoded frame in pending handshake context.
/// Returns error for any unrecoverable violation.
fn process_handshake_frame(
    conn: ConnectionId,
    frame: Frame,
    transports: &mut ActiveTransports,
    pending: &mut PendingHandshakes,
    sessions: &mut SessionRegistry,
    metrics: &mut NetMetrics,
    ev_established: &mut EventWriter<NetSessionEstablished>,
) -> anyhow::Result<()> {
    match frame {
        Frame::Handshake(HandshakeFrame::ClientHello { version, token }) => {
            // Ensure this is the first (and only) handshake frame expected.
            // If buffer still contains data, or additional handshake frames appear later,
            // they will produce errors when session path is introduced.
            // TODO(M1-HS-VERSION): Real negotiation if multiple supported.
            if !SUPPORTED_VERSIONS.contains(&version) {
                send_handshake_error(
                    conn,
                    HandshakeErrorCode::UnsupportedVersion,
                    "version",
                    transports,
                    pending.max_frame_bytes,
                );
                metrics.handshake_fail_version = metrics.handshake_fail_version.saturating_add(1);
                anyhow::bail!("unsupported version {version}");
            }

            // TODO(M1-HS-AUTH): Validate token. Current placeholder accepts everything.
            let _token = token;

            // Rate / capacity limit
            if (sessions.sessions.len() as u32) >= pending.max_sessions {
                send_handshake_error(
                    conn,
                    HandshakeErrorCode::RateLimited,
                    "capacity",
                    transports,
                    pending.max_frame_bytes,
                );
                metrics.handshake_fail_rate_limited =
                    metrics.handshake_fail_rate_limited.saturating_add(1);
                anyhow::bail!("capacity exceeded");
            }

            // Allocate session
            let ph = pending
                .map
                .get(&conn)
                .ok_or_else(|| anyhow::anyhow!("pending state vanished"))?;
            let sess_id = sessions.allocate_session_id();
            sessions.insert(sess_id, conn, ph.provider);

            // Send ServerHello
            let reply = Frame::Handshake(HandshakeFrame::ServerHello {
                session_id: sess_id,
                accepted_version: version,
            });
            if let Err(e) = send_frame_raw(conn, &reply, transports, pending.max_frame_bytes) {
                // Attempt to roll back session insertion
                // (Simplistic: leave it; future improvements can mark it "half-open" until ack)
                bevy::log::error!(
                    target: "server::net::handshake",
                    "Failed to send ServerHello conn={:?} err={e}",
                    conn
                );
                anyhow::bail!("send serverhello: {e}");
            }

            // Drop pending state and emit event
            pending.map.remove(&conn);
            ev_established.write(NetSessionEstablished(sess_id));
            metrics.handshake_success = metrics.handshake_success.saturating_add(1);

            bevy::log::info!(
                target: "server::net::handshake",
                "Handshake success conn={:?} -> session={}",
                conn,
                sess_id
            );
        }
        Frame::Handshake(HandshakeFrame::ServerHello { .. }) => {
            anyhow::bail!("unexpected ServerHello on server side")
        }
        Frame::Handshake(HandshakeFrame::HandshakeError { .. }) => {
            anyhow::bail!("unexpected HandshakeError from client")
        }
        Frame::Transport(_) => {
            // Transport frames during pending handshake are not allowed.
            send_handshake_error(
                conn,
                HandshakeErrorCode::Malformed,
                "unexpected transport frame",
                transports,
                pending.max_frame_bytes,
            );
            metrics.handshake_fail_malformed = metrics.handshake_fail_malformed.saturating_add(1);
            anyhow::bail!("transport frame before handshake complete");
        }
    }
    Ok(())
}

/// System: Timeout stale pending handshakes.
pub fn handshake_timeout_system(
    mut pending: ResMut<PendingHandshakes>,
    mut transports: ResMut<ActiveTransports>,
    mut metrics: ResMut<NetMetrics>,
) {
    if pending.map.is_empty() {
        return;
    }
    let now = Instant::now();
    let timeout = pending.timeout;
    let mut to_remove = Vec::new();

    for (conn, hs) in pending.map.iter() {
        if now.duration_since(hs.started_at) > timeout {
            bevy::log::warn!(
                target: "server::net::handshake",
                "Handshake timeout conn={:?}",
                conn
            );
            send_handshake_error(
                *conn,
                HandshakeErrorCode::Timeout,
                "timeout",
                &mut transports,
                pending.max_frame_bytes,
            );
            transports.disconnect(*conn, Some("handshake timeout"));
            to_remove.push(*conn);
            metrics.handshake_timeout = metrics.handshake_timeout.saturating_add(1);
        }
    }

    for c in to_remove {
        pending.map.remove(&c);
    }
}

/// Helper: Encode and send a single frame (uses length-prefix + codec).
fn send_frame_raw(
    conn: ConnectionId,
    frame: &Frame,
    transports: &mut ActiveTransports,
    max_frame_bytes: u32,
) -> anyhow::Result<()> {
    let codec = FrameCodec::new(max_frame_bytes);
    let mut buf = Vec::new();
    codec.encode(frame, &mut buf)?;
    transports
        .send_raw(conn, &buf)
        .map_err(|e| anyhow::anyhow!("transport send_raw: {e:?}"))?;
    Ok(())
}

/// Helper: Build and transmit a handshake error frame (best effort).
fn send_handshake_error(
    conn: ConnectionId,
    code: HandshakeErrorCode,
    msg: &str,
    transports: &mut ActiveTransports,
    max_frame_bytes: u32,
) {
    let frame = Frame::Handshake(HandshakeFrame::HandshakeError {
        code,
        message: msg.to_string(),
    });
    if let Err(e) = send_frame_raw(conn, &frame, transports, max_frame_bytes) {
        bevy::log::debug!(
            target: "server::net::handshake",
            "Failed to send HandshakeError conn={:?} err={e}",
            conn
        );
    }
}

/// Utility for plugin setup: insert default PendingHandshakes resource.
///
/// Example usage in plugin build:
/// commands.insert_resource(default_pending_handshakes(Duration::from_millis(5000), 64*1024, 10_000));
pub fn default_pending_handshakes(
    timeout: Duration,
    max_frame_bytes: u32,
    max_sessions: u32,
) -> PendingHandshakes {
    PendingHandshakes::new(timeout, max_frame_bytes, max_sessions)
}

// -------------------------------------------------------------------------------------------------
// Tests (basic logic with a fake transport)
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{ProviderEvent, ProviderKind, TransportError, TransportProvider};
    use std::sync::Arc;

    // Minimal fake provider capturing sent bytes
    struct FakeProvider {
        events: Vec<ProviderEvent>,
        sent: Vec<(ConnectionId, Vec<u8>)>,
        started: bool,
    }

    impl FakeProvider {
        fn new() -> Self {
            Self {
                events: Vec::new(),
                sent: Vec::new(),
                started: false,
            }
        }
        fn push_event(&mut self, ev: ProviderEvent) {
            self.events.push(ev);
        }
    }

    impl TransportProvider for FakeProvider {
        fn kind(&self) -> ProviderKind {
            ProviderKind::Local
        }

        fn start(&mut self, _cfg: Arc<crate::ServerRuntimeConfig>) -> anyhow::Result<()> {
            self.started = true;
            Ok(())
        }

        fn poll_events(&mut self, out: &mut Vec<ProviderEvent>) {
            out.extend(self.events.drain(..));
        }

        fn send(&mut self, id: ConnectionId, bytes: &[u8]) -> Result<(), TransportError> {
            self.sent.push((id, bytes.to_vec()));
            Ok(())
        }

        fn disconnect(&mut self, _id: ConnectionId, _reason: Option<&str>) {}

        fn shutdown(&mut self) {}

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    fn test_runtime_cfg() -> crate::ServerRuntimeConfig {
        crate::ServerRuntimeConfig {
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
    fn handshake_success_flow() {
        let mut app = App::new();
        app.add_event::<NetSessionEstablished>();
        app.insert_resource(SessionRegistry::new(100));
        app.insert_resource(default_pending_handshakes(
            Duration::from_millis(2000),
            64 * 1024,
            100,
        ));

        // Build ActiveTransports with fake provider
        use crate::transport::ActiveTransports;
        let mut active = ActiveTransports::new();
        active.register_provider(Box::new(FakeProvider::new()));
        // Simulate started
        let cfg = Arc::new(test_runtime_cfg());
        active.start_all(cfg).unwrap();

        // Inject NewConnection + RawInbound (ClientHello)
        {
            let prov = &mut active.providers_mut()[0];
            let fake = prov
                .as_any_mut()
                .downcast_mut::<FakeProvider>()
                .expect("fake");
            fake.push_event(ProviderEvent::NewConnection {
                id: ConnectionId(1),
                remote: "local".into(),
                via: ProviderKind::Local,
            });

            // Build client hello frame
            let ch = Frame::Handshake(HandshakeFrame::ClientHello {
                version: 1,
                token: None,
            });
            let mut buf = Vec::new();
            FrameCodec::new(64 * 1024).encode(&ch, &mut buf).unwrap();
            fake.push_event(ProviderEvent::RawInbound {
                id: ConnectionId(1),
                bytes: buf,
                via: ProviderKind::Local,
            });
        }

        app.insert_resource(active);

        app.add_systems(Update, handshake_process_transport_events_system);

        app.update();

        // Session established event fired?
        // (Event reader inspection removed – registry assertion below suffices for test)
        // Due to reader usage complexity in simple test, directly inspect registry
        let registry = app.world().get_resource::<SessionRegistry>().unwrap();
        assert_eq!(
            registry.sessions.len(),
            1,
            "expected one established session"
        );
    }

    #[test]
    fn unsupported_version_rejected() {
        let mut app = App::new();
        app.insert_resource(SessionRegistry::new(100));
        app.insert_resource(default_pending_handshakes(
            Duration::from_millis(2000),
            64 * 1024,
            100,
        ));
        app.add_event::<NetSessionEstablished>();

        let mut active = crate::transport::ActiveTransports::new();
        active.register_provider(Box::new(FakeProvider::new()));
        active.start_all(Arc::new(test_runtime_cfg())).unwrap();
        {
            let prov = &mut active.providers_mut()[0];
            let fake = prov.as_any_mut().downcast_mut::<FakeProvider>().unwrap();
            fake.push_event(ProviderEvent::NewConnection {
                id: ConnectionId(11),
                remote: "local".into(),
                via: ProviderKind::Local,
            });
            // Unsupported version 999
            let ch = Frame::Handshake(HandshakeFrame::ClientHello {
                version: 999,
                token: None,
            });
            let mut buf = Vec::new();
            FrameCodec::new(64 * 1024).encode(&ch, &mut buf).unwrap();
            fake.push_event(ProviderEvent::RawInbound {
                id: ConnectionId(11),
                bytes: buf,
                via: ProviderKind::Local,
            });
        }
        app.insert_resource(active);
        app.add_systems(Update, handshake_process_transport_events_system);
        app.update();

        // No sessions expected
        let registry = app.world().get_resource::<SessionRegistry>().unwrap();
        assert!(registry.sessions.is_empty(), "no session should be created");
    }

    // ----------------------------------------------------------------------------
    // Additional Tests – Event Buffer & Handshake Consumption
    // ----------------------------------------------------------------------------

    #[test]
    fn transport_event_buffer_consumed_only_once() {
        // Setup minimal Bevy app
        let mut app = App::new();
        app.add_event::<NetSessionEstablished>();
        app.insert_resource(SessionRegistry::new(100));
        app.insert_resource(default_pending_handshakes(
            Duration::from_millis(2_000),
            64 * 1024,
            100,
        ));
        app.insert_resource(NetMetrics::default());

        // Active transports with fake provider
        use crate::transport::ActiveTransports;
        let mut active = ActiveTransports::new();
        active.register_provider(Box::new(FakeProvider::new()));
        active.start_all(Arc::new(test_runtime_cfg())).unwrap();

        // Build handshake ClientHello
        let ch = Frame::Handshake(HandshakeFrame::ClientHello {
            version: 1,
            token: None,
        });
        let mut encoded = Vec::new();
        FrameCodec::new(64 * 1024)
            .encode(&ch, &mut encoded)
            .unwrap();

        // Inject events into fake provider
        {
            let prov = &mut active.providers_mut()[0];
            let fake = prov.as_any_mut().downcast_mut::<FakeProvider>().unwrap();
            fake.push_event(ProviderEvent::NewConnection {
                id: ConnectionId(42),
                remote: "local".into(),
                via: ProviderKind::Local,
            });
            fake.push_event(ProviderEvent::RawInbound {
                id: ConnectionId(42),
                bytes: encoded,
                via: ProviderKind::Local,
            });
        }

        // TransportEventBuffer + resources
        app.insert_resource(active);
        app.insert_resource(TransportEventBuffer { events: Vec::new() });

        // Simuliere poll_transports_system (rekonstruiert Events ohne Clone)
        // Simuliere poll_transports_system ohne doppelte mutable Borrows
        let reconstructed: Vec<ProviderEvent> = {
            let world = app.world_mut();
            let mut tr = world.get_resource_mut::<ActiveTransports>().unwrap();
            let slice = tr.poll();
            slice
                .iter()
                .map(|ev| match ev {
                    ProviderEvent::NewConnection { id, remote, via } => {
                        ProviderEvent::NewConnection {
                            id: *id,
                            remote: remote.clone(),
                            via: *via,
                        }
                    }
                    ProviderEvent::Disconnected { id, reason, via } => {
                        ProviderEvent::Disconnected {
                            id: *id,
                            reason: reason.clone(),
                            via: *via,
                        }
                    }
                    ProviderEvent::RawInbound { id, bytes, via } => ProviderEvent::RawInbound {
                        id: *id,
                        bytes: bytes.clone(),
                        via: *via,
                    },
                })
                .collect()
        };
        {
            let world = app.world_mut();
            let mut buf = world.get_resource_mut::<TransportEventBuffer>().unwrap();
            buf.events.extend(reconstructed);
        }

        // Register handshake system only
        app.add_systems(Update, handshake_process_transport_events_system);

        // First update: should consume buffer & establish session
        app.update();

        // Buffer leer?
        let buf = app.world().get_resource::<TransportEventBuffer>().unwrap();
        assert!(
            buf.events.is_empty(),
            "Event buffer should be empty after handshake system ran"
        );

        // Session angelegt?
        let reg = app.world().get_resource::<SessionRegistry>().unwrap();
        assert_eq!(reg.sessions.len(), 1, "expected exactly one session");

        // Second update: no duplicate session
        app.update();
        let reg2 = app.world().get_resource::<SessionRegistry>().unwrap();
        assert_eq!(
            reg2.sessions.len(),
            1,
            "handshake system must not process same events twice"
        );
    }

    #[test]
    fn raw_inbound_ignored_when_not_pending() {
        let mut app = App::new();
        app.add_event::<NetSessionEstablished>();
        app.insert_resource(SessionRegistry::new(100));
        app.insert_resource(default_pending_handshakes(
            Duration::from_millis(2_000),
            64 * 1024,
            100,
        ));
        app.insert_resource(NetMetrics::default());

        // Active transports + fake
        let mut active = ActiveTransports::new();
        active.register_provider(Box::new(FakeProvider::new()));
        active.start_all(Arc::new(test_runtime_cfg())).unwrap();

        // Inject ONLY RawInbound without NewConnection (simulate spurious data)
        {
            let prov = &mut active.providers_mut()[0];
            let fake = prov.as_any_mut().downcast_mut::<FakeProvider>().unwrap();
            fake.push_event(ProviderEvent::RawInbound {
                id: ConnectionId(777),
                bytes: b"junk".to_vec(),
                via: ProviderKind::Local,
            });
        }

        app.insert_resource(active);
        app.insert_resource(TransportEventBuffer { events: Vec::new() });

        // Simulated poll
        let reconstructed: Vec<ProviderEvent> = {
            let world = app.world_mut();
            let mut tr = world.get_resource_mut::<ActiveTransports>().unwrap();
            let slice = tr.poll();
            slice
                .iter()
                .filter_map(|ev| match ev {
                    ProviderEvent::RawInbound { id, bytes, via } => {
                        Some(ProviderEvent::RawInbound {
                            id: *id,
                            bytes: bytes.clone(),
                            via: *via,
                        })
                    }
                    _ => None,
                })
                .collect()
        };
        {
            let world = app.world_mut();
            let mut buf = world.get_resource_mut::<TransportEventBuffer>().unwrap();
            buf.events.extend(reconstructed);
        }

        app.add_systems(Update, handshake_process_transport_events_system);
        app.update();

        // Keine Session entstanden
        let reg = app.world().get_resource::<SessionRegistry>().unwrap();
        assert!(
            reg.sessions.is_empty(),
            "no session should be created from stray RawInbound without pending handshake"
        );
    }
}
