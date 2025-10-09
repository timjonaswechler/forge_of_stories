use shared::transport::{ClientTransport, TransportPayload, TransportResult};
use shared::{
    ClientEvent, TransportCapabilities, TransportError,
    channels::ChannelKind,
    steam::{SteamAppId, SteamAuthTicket},
};

use crate::transport::ConnectTarget;

#[derive(thiserror::Error, Debug)]
pub enum SteamTransportError {
    #[error("steamworks feature is disabled")]
    Disabled,
}

impl From<SteamTransportError> for TransportError {
    fn from(_: SteamTransportError) -> Self {
        TransportError::Other("steamworks feature is disabled".into())
    }
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

impl ClientTransport for SteamClientTransport {
    type ConnectTarget = ConnectTarget;

    fn poll_events(&mut self, _output: &mut Vec<ClientEvent>) {}

    fn connect(&mut self, _target: Self::ConnectTarget) -> TransportResult<()> {
        Err(SteamTransportError::Disabled.into())
    }

    fn disconnect(&mut self) -> TransportResult<()> {
        Err(SteamTransportError::Disabled.into())
    }

    fn send(&mut self, _payload: TransportPayload) -> TransportResult<()> {
        Err(SteamTransportError::Disabled.into())
    }
}

impl SteamClientTransport {
    pub fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities::default()
    }
}
