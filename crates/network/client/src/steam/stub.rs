use bytes::Bytes;
use network_shared::{
    steam::SteamAppId,
    ClientEvent, DisconnectReason, OutgoingMessage, TransportCapabilities,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::transport::{ClientTransport, ConnectTarget};

#[derive(thiserror::Error, Debug)]
pub enum SteamTransportError {
    #[error("steamworks feature is disabled")]
    Disabled,
}

#[derive(Debug)]
pub struct SteamClientTransport;

impl SteamClientTransport {
    pub fn new_default() -> Result<Self, SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }

    pub fn new(_app_id: SteamAppId) -> Result<Self, SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }
}

impl ClientTransport for SteamClientTransport {
    type Error = SteamTransportError;

    fn connect(
        &mut self,
        _target: ConnectTarget,
        _events: UnboundedSender<ClientEvent>,
    ) -> Result<(), Self::Error> {
        Err(SteamTransportError::Disabled)
    }

    fn disconnect(&mut self, _reason: DisconnectReason) -> Result<(), Self::Error> {
        Err(SteamTransportError::Disabled)
    }

    fn send(&self, _message: OutgoingMessage) -> Result<(), Self::Error> {
        Err(SteamTransportError::Disabled)
    }

    fn send_datagram(&self, _payload: Bytes) -> Result<(), Self::Error> {
        Err(SteamTransportError::Disabled)
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities::default()
    }
}
