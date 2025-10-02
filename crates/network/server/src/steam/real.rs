use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use network_shared::{
    channels::{ChannelId, ChannelKind},
    steam::{SteamAppId, STEAM_APP_ID},
    ClientId,
    DisconnectReason,
    OutgoingMessage,
    TransportCapabilities,
    TransportEvent,
};
use steamworks::{networking_sockets::NetworkingSockets, Client, ClientManager, Server, ServerManager, SteamError};
use tokio::{
    sync::{mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
    time::sleep,
};

use crate::transport::ServerTransport;

#[derive(thiserror::Error, Debug)]
pub enum SteamServerTransportError {
    #[error("steamworks initialization failed: {0}")]
    Init(#[from] SteamError),
    #[error("steam transport not yet implemented")]
    Unimplemented,
}

#[derive(Debug)]
pub struct SteamServerTransport {
    app_id: SteamAppId,
    server: Arc<Server<ServerManager>>,
    callbacks: Option<JoinHandle<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    capabilities: TransportCapabilities,
    unreliable_channel: Option<ChannelId>,
}

impl SteamServerTransport {
    pub fn new_default(channels: &[ChannelKind]) -> Result<Self, SteamServerTransportError> {
        Self::new(SteamAppId::development(), channels)
    }

    pub fn new(
        app_id: SteamAppId,
        channels: &[ChannelKind],
    ) -> Result<Self, SteamServerTransportError> {
        let ip = std::net::Ipv4Addr::UNSPECIFIED;
        let port = 0;
        let game_port = 0;
        let query_port = 0;
        let (server, single) = Server::init_app(app_id.0, ip, port, game_port, query_port, &[], None)?;
        let server = Arc::new(server);
        let single = Arc::new(single);
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let server_clone = Arc::clone(&server);
        let single_clone = Arc::clone(&single);
        let callbacks = tokio::spawn(async move {
            loop {
                single_clone.run_callbacks(&server_clone);
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }
                sleep(Duration::from_millis(16)).await;
            }
        });

        let unreliable_channel = channels.iter().enumerate().find_map(|(idx, kind)| {
            if matches!(kind, ChannelKind::Unreliable) {
                Some(idx as ChannelId)
            } else {
                None
            }
        });

        Ok(Self {
            app_id,
            server,
            callbacks: Some(callbacks),
            shutdown: Some(shutdown_tx),
            capabilities: TransportCapabilities::default(),
            unreliable_channel,
        })
    }
}

impl ServerTransport for SteamServerTransport {
    type Error = SteamServerTransportError;

    fn start(&mut self, _events: UnboundedSender<TransportEvent>) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Unimplemented)
    }

    fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(handle) = self.callbacks.take() {
            handle.abort();
        }
    }

    fn send(&self, _client: ClientId, _message: OutgoingMessage) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Unimplemented)
    }

    fn send_datagram(&self, _client: ClientId, _payload: Bytes) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Unimplemented)
    }

    fn disconnect(&self, _client: ClientId, _reason: DisconnectReason) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Unimplemented)
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }
}

impl Drop for SteamServerTransport {
    fn drop(&mut self) {
        self.stop();
    }
}
