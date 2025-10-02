use bytes::Bytes;
use network_shared::{
    channels::ChannelKind,
    steam::SteamAppId,
    ClientId, DisconnectReason, OutgoingMessage, TransportCapabilities, TransportEvent,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::transport::ServerTransport;

#[derive(thiserror::Error, Debug)]
pub enum SteamServerTransportError {
    #[error("steamworks feature is disabled")]
    Disabled,
}

#[derive(Debug)]
pub struct SteamServerTransport;

impl SteamServerTransport {
    pub fn new_default(_channels: &[ChannelKind]) -> Result<Self, SteamServerTransportError> {
        Err(SteamServerTransportError::Disabled)
    }

    pub fn new(
        _app_id: SteamAppId,
        _channels: &[ChannelKind],
    ) -> Result<Self, SteamServerTransportError> {
        Err(SteamServerTransportError::Disabled)
    }
}

impl ServerTransport for SteamServerTransport {
    type Error = SteamServerTransportError;

    fn start(&mut self, _events: UnboundedSender<TransportEvent>) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Disabled)
    }

    fn stop(&mut self) {}

    fn send(&self, _client: ClientId, _message: OutgoingMessage) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Disabled)
    }

    fn send_datagram(&self, _client: ClientId, _payload: Bytes) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Disabled)
    }

    fn disconnect(&self, _client: ClientId, _reason: DisconnectReason) -> Result<(), Self::Error> {
        Err(SteamServerTransportError::Disabled)
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities::default()
    }
}
