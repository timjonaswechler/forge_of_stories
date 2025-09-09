/*!
QUIC Transport Provider Skeleton

Ziel:
- Implementiert das `TransportProvider` Trait für QUIC (quinn) als austauschbare Transport-Schicht.
- Dient als Brücke zwischen Low-Level QUIC Accept Loop und höherem Protokoll-/Handshake-Layer.
- Noch kein Frame-Handling, kein Senden von Nutzdaten – Fokus: grundlegendes Connection-Akzeptieren + Events.

Aktueller Umfang (Skeleton):
- Startet einen QUIC Listener (Endpoint) mit TLS (nutzt bestehenden TLS Loader).
- Spawnt eine Tokio-Task, die eingehende Verbindungen akzeptiert und `ProviderEvent::NewConnection` ausgibt.
- Speichert Verbindungsobjekte noch nicht (Senden / Disconnect sind Platzhalter).
- Polling liefert gesammelte Events non-blocking an die obere Ebene.

Geplante Erweiterungen (Folgeschritte):
1. Speicherung akzeptierter `quinn::Connection` Objekte (HashMap<ConnectionId, Connection>).
2. Erster Stream (BiDi) öffnen / akzeptieren → Rohdaten als `RawInbound`.
3. Handshake State Machine (Version / Token) an Protokoll-Layer übergeben.
4. Senden implementieren (Stream-Selektion / Outbound Frame Encoding).
5. Graceful Disconnect (inkl. Close Reason Codes).
6. Optional: Endpoint Shutdown Signalisierung (Drop Guard / Shutdown Channel).
7. Metrics Hooks (connection_accept_total, connection_close_total, accept_errors).

Hinweise zur QUIC / rustls Integration:
- Der TLS Loader liefert aktuell `Arc<rustls::ServerConfig>`.
- Quinn 0.11.x erfordert einen kompatiblen Crypto-Wrapper. Abhängig von aktivierten Features kann
  `quinn::ServerConfig::with_crypto(Arc<rustls::ServerConfig>)` funktionieren oder ein Adapter
  (z. B. `quinn::crypto::rustls::ServerConfig`) benötigt werden. Falls Build-Fehler auftreten:
  => TODO(M1-QUIC): Anpassung an die tatsächlich aktivierten quinn/rustls Features vornehmen.

Sicherheitsaspekte (später):
- Session-Limits (max_sessions)
- Rate Limiting (Accept Burst Begrenzung)
- ALPN Validierung / Versionsverhandlung (Handshake-Layer)
- Logging sensibler Daten vermeiden

(C) Forge of Stories – Network Architecture Evolution (M1)
*/

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

use tokio::task::JoinHandle;

use crate::{
    ServerRuntimeConfig,
    quic::tls::load_or_generate_tls,
    transport::{ConnectionId, ProviderEvent, ProviderKind, TransportError, TransportProvider},
};

/// QUIC Provider – verwaltet Endpoint + Accept Loop Task.
pub struct QuicProvider {
    endpoint: Option<quinn::Endpoint>,
    // Accept loop task (spawns per-connection read tasks).
    accept_task: Option<JoinHandle<()>>,
    // Event channel (worker tasks -> provider poll).
    evt_tx: mpsc::Sender<ProviderEvent>,
    evt_rx: Mutex<mpsc::Receiver<ProviderEvent>>,
    // Connection map (for send / disconnect)
    connections: Arc<Mutex<HashMap<ConnectionId, quinn::Connection>>>,
    // Laufende ID-Vergabe
    next_id: Arc<AtomicU64>,
}

impl Default for QuicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl QuicProvider {
    pub fn new() -> Self {
        let (evt_tx, evt_rx) = mpsc::channel();
        Self {
            endpoint: None,
            accept_task: None,
            evt_tx,
            evt_rx: Mutex::new(evt_rx),
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    fn emit(&self, ev: ProviderEvent) {
        // Dropped receiver => still ignore (shutdown path)
        let _ = self.evt_tx.send(ev);
    }

    pub(crate) fn allocate_conn_id(&self) -> ConnectionId {
        ConnectionId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

impl TransportProvider for QuicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Quic
    }

    fn start(&mut self, cfg: Arc<ServerRuntimeConfig>) -> anyhow::Result<()> {
        if self.endpoint.is_some() {
            return Ok(()); // bereits gestartet
        }

        // 1) TLS laden (oder generieren im debug)
        let tls_cfg =
            load_or_generate_tls(&cfg).map_err(|e| anyhow::anyhow!("tls load failed: {e}"))?;

        // 2) Quinn ServerConfig bauen
        // HINWEIS: Falls dieser Aufruf scheitert (Trait Bound), an passende quinn-rustls Adapter anpassen.
        let mut server_config = {
            use std::convert::TryFrom;
            let base = Arc::new(tls_cfg);
            let crypto =
                quinn::crypto::rustls::QuicServerConfig::try_from(base.clone()).map_err(|_| {
                    anyhow::anyhow!(
                        "invalid TLS server config for QUIC (missing TLS1.3 or cipher suites)"
                    )
                })?;
            quinn::ServerConfig::with_crypto(Arc::new(crypto))
        };

        // Transportkonfiguration (aus RuntimeConfig Snapshot übernommen)
        let mut transport = quinn::TransportConfig::default();
        // Stream Limits
        transport
            .max_concurrent_bidi_streams(quinn::VarInt::from_u32(cfg.max_concurrent_bidi_streams));
        transport
            .max_concurrent_uni_streams(quinn::VarInt::from_u32(cfg.max_concurrent_uni_streams));
        // Keep-Alive (nur setzen wenn > 0)
        if cfg.keep_alive_interval_ms > 0 {
            transport.keep_alive_interval(Some(std::time::Duration::from_millis(
                cfg.keep_alive_interval_ms,
            )));
        }
        // Idle Timeout (cfg.max_idle_timeout_ms kann größer als u32 sein → clamp)
        let idle_ms: u32 = cfg.max_idle_timeout_ms.min(u64::from(u32::MAX) as u64) as u32;
        transport.max_idle_timeout(Some(quinn::IdleTimeout::from(quinn::VarInt::from_u32(
            idle_ms,
        ))));
        // (Optionale TODOs: initial_congestion_window, mtu, client_ip_migration, zero_rtt_resumption, qos_traffic_prioritization, nat_traversal –
        // werden hier noch nicht direkt von quinn exposed oder benötigen erweitertes Mapping.)

        server_config.transport_config(Arc::new(transport));

        // 3) Bind-Adresse
        let addr: SocketAddr = format!("{}:{}", cfg.bind_addr, cfg.port).parse()?;

        // 4) Endpoint erstellen
        let endpoint = quinn::Endpoint::server(server_config, addr)
            .map_err(|e| anyhow::anyhow!("endpoint bind failed: {e}"))?;

        // 5) Accept Loop Task (quinn 0.11: pro Verbindung erneut `endpoint.accept().await`)
        let evt_tx = self.evt_tx.clone();
        let id_src = self.next_id.clone();
        let endpoint_clone = endpoint.clone();
        let connections = self.connections.clone();
        let accept_handle = tokio::spawn(async move {
            loop {
                let incoming_opt = endpoint_clone.accept().await;
                let Some(incoming) = incoming_opt else {
                    break;
                };
                let tx_new = evt_tx.clone();
                let id_src_inner = id_src.clone();
                let connections_inner = connections.clone();
                tokio::spawn(async move {
                    match incoming.await {
                        Ok(connection) => {
                            let conn_id =
                                ConnectionId(id_src_inner.fetch_add(1, Ordering::Relaxed));
                            let remote = connection.remote_address().to_string();
                            {
                                let mut guard = connections_inner.lock().unwrap();
                                guard.insert(conn_id, connection.clone());
                            }
                            let _ = tx_new.send(ProviderEvent::NewConnection {
                                id: conn_id,
                                remote,
                                via: ProviderKind::Quic,
                            });
                            // Spawn read loop for incoming BiDi streams
                            let tx_read = tx_new.clone();
                            let conn_for_reads = connection.clone();
                            tokio::spawn(async move {
                                loop {
                                    match conn_for_reads.accept_bi().await {
                                        Ok((_, mut recv_stream)) => {
                                            // We do not use send_stream yet (reserved for future responses)
                                            tokio::spawn({
                                                let tx_raw = tx_read.clone();
                                                async move {
                                                    // Stream read loop
                                                    loop {
                                                        match recv_stream
                                                            .read_chunk(16 * 1024, true)
                                                            .await
                                                        {
                                                            Ok(Some(chunk)) => {
                                                                if !chunk.bytes.is_empty() {
                                                                    let _ = tx_raw.send(
                                                                        ProviderEvent::RawInbound {
                                                                            id: conn_id,
                                                                            bytes: chunk
                                                                                .bytes
                                                                                .to_vec(),
                                                                            via: ProviderKind::Quic,
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                            Ok(None) => {
                                                                // EOF
                                                                let _ = tx_raw.send(
                                                                    ProviderEvent::Disconnected {
                                                                        id: conn_id,
                                                                        reason: Some(
                                                                            "stream closed".into(),
                                                                        ),
                                                                        via: ProviderKind::Quic,
                                                                    },
                                                                );
                                                                break;
                                                            }
                                                            Err(e) => {
                                                                let _ = tx_raw.send(
                                                                    ProviderEvent::Disconnected {
                                                                        id: conn_id,
                                                                        reason: Some(format!(
                                                                            "stream error: {e}"
                                                                        )),
                                                                        via: ProviderKind::Quic,
                                                                    },
                                                                );
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        Err(e) => {
                                            // Accept failed – treat as connection closed
                                            let _ = tx_new.send(ProviderEvent::Disconnected {
                                                id: conn_id,
                                                reason: Some(format!("accept_bi error: {e}")),
                                                via: ProviderKind::Quic,
                                            });
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            let _ = tx_new.send(ProviderEvent::Disconnected {
                                id: ConnectionId(0),
                                reason: Some(format!("connect error: {e}")),
                                via: ProviderKind::Quic,
                            });
                        }
                    }
                });
            }
        });

        self.accept_task = Some(accept_handle);
        self.endpoint = Some(endpoint);

        bevy::log::info!(
            target: "server::net::quic",
            "QuicProvider gestartet: {}:{}",
            cfg.bind_addr,
            cfg.port
        );
        Ok(())
    }

    fn poll_events(&mut self, out: &mut Vec<ProviderEvent>) {
        // Ziehe alle verfügbaren Events non-blocking
        let rx_guard = self.evt_rx.lock().unwrap();
        while let Ok(ev) = rx_guard.try_recv() {
            out.push(ev);
        }
    }

    fn send(&mut self, id: ConnectionId, bytes: &[u8]) -> Result<(), TransportError> {
        let conn_opt = {
            let guard = self.connections.lock().unwrap();
            guard.get(&id).cloned()
        };
        let Some(conn) = conn_opt else {
            return Err(TransportError::UnknownConnection);
        };
        let data = bytes.to_vec();
        // Spawn async send task (non-blocking)
        tokio::spawn(async move {
            if let Ok(mut stream) = conn.open_uni().await {
                if let Err(e) = stream.write_all(&data).await {
                    bevy::log::debug!(
                        target:"server::net::quic",
                        "send failed conn={:?}: {e}",
                        id
                    );
                } else {
                    let _ = stream.finish();
                }
            }
        });
        Ok(())
    }

    fn disconnect(&mut self, id: ConnectionId, reason: Option<&str>) {
        let conn = {
            let mut guard = self.connections.lock().unwrap();
            guard.remove(&id)
        };
        if let Some(c) = conn {
            let msg = reason.unwrap_or("disconnect");
            c.close(0u32.into(), msg.as_bytes());
        }
        bevy::log::debug!(
            target: "server::net::quic",
            "disconnect() executed: id={:?} reason={:?}",
            id,
            reason
        );
    }

    fn shutdown(&mut self) {
        // Close all active connections
        {
            let mut guard = self.connections.lock().unwrap();
            for (cid, conn) in guard.drain() {
                conn.close(0u32.into(), b"server shutdown");
                bevy::log::trace!(
                    target:"server::net::quic",
                    "connection closed during shutdown id={:?}",
                    cid
                );
            }
        }
        if let Some(ep) = self.endpoint.take() {
            ep.close(0u32.into(), b"shutdown");
        }
        if let Some(h) = self.accept_task.take() {
            tokio::spawn(async move {
                let _ = h.await;
            });
        }
        bevy::log::info!(
            target: "server::net::quic",
            "QuicProvider shutdown abgeschlossen"
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Helper: Erzeugt einen QuicProvider in Box-Form.
pub fn boxed_quic_provider() -> Box<dyn TransportProvider> {
    Box::new(QuicProvider::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    // removed unused import: ServerRuntimeConfig

    #[test]
    fn id_allocation_monotonic() {
        let p = QuicProvider::new();
        let a = p.allocate_conn_id();
        let b = p.allocate_conn_id();
        assert!(b.0 > a.0);
    }

    #[test]
    fn provider_kind() {
        let p = QuicProvider::new();
        assert_eq!(p.kind(), ProviderKind::Quic);
    }

    #[test]
    fn skeleton_send_unimplemented() {
        let mut p = QuicProvider::new();
        let res = p.send(ConnectionId(1), b"abc");
        assert!(res.is_err());
    }

    // Hinweis: Start-Test würde real TLS laden; hier übersprungen (Integrationstest sinnvoller)
    #[test]
    fn can_construct_boxed() {
        let _p: Box<dyn TransportProvider> = boxed_quic_provider();
    }

    // Test-helper specific impl no longer needed (allocate_conn_id is now pub(crate))
}
