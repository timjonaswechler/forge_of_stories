use serde::{Deserialize, Serialize};

use crate::protocol;

/// A lightweight message envelope used to attach minimal metadata (e.g. sequencing, timestamp)
/// to typed payloads. The payload type is generic so you can carry domain messages and later
/// convert them to wire protocol enums (Rpc/Event).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// A monotonically increasing identifier (caller-provided).
    pub id: u64,
    /// Monotonic timestamp in milliseconds (caller-provided).
    /// This is intentionally not a wall-clock time to avoid clock skew issues.
    pub ts_mono_ms: u64,
    /// The message payload.
    pub payload: T,
}

impl<T> Envelope<T> {
    pub fn new(id: u64, ts_mono_ms: u64, payload: T) -> Self {
        Self {
            id,
            ts_mono_ms,
            payload,
        }
    }

    /// Transform the payload while preserving metadata.
    pub fn map_payload<U>(self, f: impl FnOnce(T) -> U) -> Envelope<U> {
        Envelope {
            id: self.id,
            ts_mono_ms: self.ts_mono_ms,
            payload: f(self.payload),
        }
    }

    /// Convert this envelope into one carrying a protocol Rpc, if the payload can be converted.
    pub fn into_rpc(self) -> Envelope<protocol::Rpc>
    where
        T: Into<protocol::Rpc>,
    {
        self.map_payload(Into::into)
    }

    /// Convert this envelope into one carrying a protocol Event, if the payload can be converted.
    pub fn into_event(self) -> Envelope<protocol::Event>
    where
        T: Into<protocol::Event>,
    {
        self.map_payload(Into::into)
    }
}

/// Convenience type aliases for envelopes already mapped to protocol enums.
pub type RpcEnvelope = Envelope<protocol::Rpc>;
pub type EventEnvelope = Envelope<protocol::Event>;

/// A unified wrapper for either direction, useful when a single queue needs to carry both.
/// This is optional; use only if convenient in your plumbing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyMessage {
    Rpc(protocol::Rpc),
    Event(protocol::Event),
}

impl From<protocol::Rpc> for AnyMessage {
    fn from(v: protocol::Rpc) -> Self {
        AnyMessage::Rpc(v)
    }
}

impl From<protocol::Event> for AnyMessage {
    fn from(v: protocol::Event) -> Self {
        AnyMessage::Event(v)
    }
}

impl Envelope<AnyMessage> {
    /// Helper to build a unified envelope from a typed one, if that payload can become AnyMessage.
    pub fn from_typed<T>(env: Envelope<T>) -> Self
    where
        T: Into<AnyMessage>,
    {
        env.map_payload(Into::into)
    }
}
