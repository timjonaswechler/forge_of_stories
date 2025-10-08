use shared::TransportError;

#[derive(Debug)]
pub enum ServerError {
    ExternalTransportFailed(TransportError),
}
