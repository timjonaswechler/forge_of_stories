use bytes::Bytes;

use crate::channels::ChannelId;

/// Opaque message wrapper passed from gameplay into the transport layer.
#[derive(Debug, Clone)]
pub struct OutgoingMessage {
    pub channel: ChannelId,
    pub payload: Bytes,
}

impl OutgoingMessage {
    pub fn new(channel: ChannelId, payload: impl Into<Bytes>) -> Self {
        Self {
            channel,
            payload: payload.into(),
        }
    }
}
