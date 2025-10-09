use shared::transport::{ServerTransport, TransportPayload, TransportResult};
use shared::{
    ClientId, TransportError, TransportEvent,
    channels::ChannelKind,
    steam::{SteamAppId, SteamAuthTicket},
};

#[derive(thiserror::Error, Debug)]
pub enum SteamServerTransportError {
    #[error("steamworks feature is disabled")]
    Disabled,
    #[error("steam authentication validation failed: {0}")]
    AuthValidation(String),
}

impl From<SteamServerTransportError> for TransportError {
    fn from(err: SteamServerTransportError) -> Self {
        match err {
            SteamServerTransportError::Disabled => {
                TransportError::Other("steamworks feature is disabled".into())
            }
            SteamServerTransportError::AuthValidation(reason) => {
                TransportError::Other(format!("steam auth validation failed: {reason}"))
            }
        }
    }
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

    pub fn validate_auth_ticket(
        &self,
        _client: ClientId,
        _ticket: SteamAuthTicket,
    ) -> Result<(), SteamServerTransportError> {
        Err(SteamServerTransportError::Disabled)
    }
}

impl ServerTransport for SteamServerTransport {
    fn poll_events(&mut self, _output: &mut Vec<TransportEvent>) {}

    fn send(&mut self, _client: ClientId, _payload: TransportPayload) -> TransportResult<()> {
        Err(SteamServerTransportError::Disabled.into())
    }

    fn broadcast(&mut self, _payload: TransportPayload) -> TransportResult<()> {
        Err(SteamServerTransportError::Disabled.into())
    }

    fn broadcast_excluding(
        &mut self,
        _exclude: &[ClientId],
        _payload: TransportPayload,
    ) -> TransportResult<()> {
        Err(SteamServerTransportError::Disabled.into())
    }
}
