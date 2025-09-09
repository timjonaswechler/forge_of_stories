/*!
Steam Transport Provider (Stub) – Forge of Stories

Goal (current state):
- Provide a placeholder implementation of the `TransportProvider` trait for a future
  Steam Relay / Steam Networking integration.
- Allow early Mode-Dispatch registration (e.g. network.mode = relay|hybrid) without
  blocking compilation or runtime.
- Emit only minimal lifecycle events (start / shutdown logs). No real connections.

Future Milestones (not implemented here):
- Integration with a Steamworks / GameNetworkingSockets binding (e.g. `steamworks` crate).
- Authentication & ownership validation (SteamID).
- Relay connection establishment (lobby / friend join flows).
- Real event emission: NewConnection / RawInbound / Disconnected.
- Graceful shutdown with session handoff / draining.
- Metrics: relay_connection_total, relay_bytes_in/out, relay_errors.

Design Notes:
- This stub returns no events and rejects all send/disconnect calls with Internal/Unknown errors.
- It can be safely registered alongside other providers (QUIC) in hybrid mode.
- Once the real implementation matures, replace internals while preserving the public helper
  `boxed_steam_provider()` and the struct name for minimal churn.

Logging Target Recommendations:
- server::net::transport (generic lifecycle)
- server::net::steam      (steam specific, once implemented)

Searchable TODO tags:
- TODO(STEAM-IMPL): Real transport logic
- TODO(STEAM-AUTH): Ownership / ticket validation
- TODO(STEAM-METRICS): Metrics counters & gauges
- TODO(STEAM-DATA): Raw inbound byte routing

(C) Forge of Stories
*/

use std::any::Any;
use std::sync::{Mutex, mpsc};

use crate::ServerRuntimeConfig;
use crate::transport::{
    ConnectionId, ProviderEvent, ProviderKind, TransportError, TransportProvider,
};

/// SteamProvider (stub).
///
/// Internal State:
/// - `started`: simple flag so multiple `start()` invocations are idempotent.
/// - `evt_rx`: prepared receiver for future async event injection (currently unused).
pub struct SteamProvider {
    started: bool,
    evt_tx: mpsc::Sender<ProviderEvent>,
    evt_rx: Mutex<mpsc::Receiver<ProviderEvent>>,
}

impl SteamProvider {
    /// Create a new stub instance.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            started: false,
            evt_tx: tx,
            evt_rx: Mutex::new(rx),
        }
    }

    #[allow(dead_code)]
    fn emit(&self, ev: ProviderEvent) {
        let _ = self.evt_tx.send(ev);
    }
}

impl Default for SteamProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportProvider for SteamProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Steam
    }

    fn start(&mut self, _cfg: std::sync::Arc<ServerRuntimeConfig>) -> anyhow::Result<()> {
        if self.started {
            return Ok(());
        }
        self.started = true;
        bevy::log::info!(
            target: "server::net::steam",
            "SteamProvider (stub) started – no real networking yet"
        );
        Ok(())
    }

    fn poll_events(&mut self, out: &mut Vec<ProviderEvent>) {
        // Drain all pending events (none generated in stub).
        let rx = self.evt_rx.lock().unwrap();
        while let Ok(ev) = rx.try_recv() {
            out.push(ev);
        }
    }

    fn send(&mut self, _id: ConnectionId, _bytes: &[u8]) -> Result<(), TransportError> {
        Err(TransportError::Internal(
            "SteamProvider stub: send() not implemented (TODO(STEAM-IMPL))".into(),
        ))
    }

    fn disconnect(&mut self, id: ConnectionId, _reason: Option<&str>) {
        bevy::log::debug!(
            target:"server::net::steam",
            "SteamProvider stub: disconnect request for {:?} (ignored)",
            id
        );
    }

    fn shutdown(&mut self) {
        if self.started {
            bevy::log::info!(
                target:"server::net::steam",
                "SteamProvider (stub) shutdown"
            );
            self.started = false;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Helper to box the provider (consistent with other provider factory patterns).
pub fn boxed_steam_provider() -> Box<dyn TransportProvider> {
    Box::new(SteamProvider::new())
}

// -------------------------------------------------------------------------------------------------
// Tests (structural)
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{ProviderKind, TransportProvider};
    use std::sync::Arc;

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
    fn kind_is_steam() {
        let p = SteamProvider::new();
        assert_eq!(p.kind(), ProviderKind::Steam);
    }

    #[test]
    fn start_is_idempotent() {
        let mut p = SteamProvider::new();
        p.start(Arc::new(dummy_cfg())).unwrap();
        assert!(p.started);
        // second start should not panic / change state
        p.start(Arc::new(dummy_cfg())).unwrap();
        assert!(p.started);
    }

    #[test]
    fn poll_empty() {
        let mut p = SteamProvider::new();
        p.start(Arc::new(dummy_cfg())).unwrap();
        let mut out = Vec::new();
        p.poll_events(&mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn send_unimplemented() {
        let mut p = SteamProvider::new();
        p.start(Arc::new(dummy_cfg())).unwrap();
        let res = p.send(ConnectionId(1), b"abc");
        assert!(res.is_err());
    }

    #[test]
    fn boxed_helper() {
        let _b: Box<dyn TransportProvider> = boxed_steam_provider();
    }
}
