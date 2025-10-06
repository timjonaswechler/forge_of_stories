use bytes::Bytes;
use network_shared::{
    ClientEvent, DisconnectReason, OutgoingMessage, TransportCapabilities,
    channels::ChannelKind,
    steam::{SteamAppId, SteamAuthTicket},
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
    pub fn new_default(_channels: &[ChannelKind]) -> Result<Self, SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }

    pub fn new(
        _app_id: SteamAppId,
        _channels: &[ChannelKind],
    ) -> Result<Self, SteamTransportError> {
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

impl SteamClientTransport {
    pub fn request_lobby_list(&self) -> Result<(), SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }

    pub fn request_auth_ticket(&self) -> Result<(), SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }

    pub fn cancel_auth_ticket(&self) {}

    pub fn has_active_auth_ticket(&self) -> bool {
        false
    }

    pub fn submit_auth_ticket(&self, _ticket: SteamAuthTicket) -> Result<(), SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }
}
