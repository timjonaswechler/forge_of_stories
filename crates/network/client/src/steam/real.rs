use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use network_shared::{
    steam::{SteamAppId, STEAM_APP_ID},
    ClientEvent, DisconnectReason, OutgoingMessage, TransportCapabilities,
};
use steamworks::{Client, ClientManager, SingleClient, SteamError};
use tokio::{
    sync::{mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
    time::sleep,
};

use crate::transport::{ClientTransport, ConnectTarget};

/// Error type for Steam transports.
#[derive(thiserror::Error, Debug)]
pub enum SteamTransportError {
    #[error("steamworks initialization failed: {0}")]
    Init(#[from] SteamError),
    #[error("steam transport not yet implemented")]
    Unimplemented,
}

#[derive(Debug)]
pub struct SteamClientTransport {
    app_id: SteamAppId,
    client: Arc<Client<ClientManager>>,
    single: Arc<SingleClient<ClientManager>>,
    callbacks: Option<JoinHandle<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    capabilities: TransportCapabilities,
}

impl SteamClientTransport {
    pub fn new_default() -> Result<Self, SteamTransportError> {
        Self::new(SteamAppId::development())
    }

    pub fn new(app_id: SteamAppId) -> Result<Self, SteamTransportError> {
        let (client, single) = steamworks::Client::init_app(app_id.0)?;
        let client = Arc::new(client);
        let single = Arc::new(single);

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let callbacks_client = Arc::clone(&client);
        let callbacks_single = Arc::clone(&single);
        let callbacks = tokio::spawn(async move {
            loop {
                callbacks_single.run_callbacks(&callbacks_client);
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }
                sleep(Duration::from_millis(16)).await;
            }
        });

        Ok(Self {
            app_id,
            client,
            single,
            callbacks: Some(callbacks),
            shutdown: Some(shutdown_tx),
            capabilities: TransportCapabilities::default(),
        })
    }
}

impl ClientTransport for SteamClientTransport {
    type Error = SteamTransportError;

    fn connect(
        &mut self,
        _target: ConnectTarget,
        _events: UnboundedSender<ClientEvent>,
    ) -> Result<(), Self::Error> {
        Err(SteamTransportError::Unimplemented)
    }

    fn disconnect(&mut self, _reason: DisconnectReason) -> Result<(), Self::Error> {
        Err(SteamTransportError::Unimplemented)
    }

    fn send(&self, _message: OutgoingMessage) -> Result<(), Self::Error> {
        Err(SteamTransportError::Unimplemented)
    }

    fn send_datagram(&self, _payload: Bytes) -> Result<(), Self::Error> {
        Err(SteamTransportError::Unimplemented)
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }
}

impl Drop for SteamClientTransport {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(handle) = self.callbacks.take() {
            handle.abort();
        }
    }
}
