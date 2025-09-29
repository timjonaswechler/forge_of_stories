//! Nachrichteneinhüllung, Sequenzierung und Fragmentierung.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Header, der jeder transportierten Nachricht vorangestellt wird.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PacketHeader {
    pub channel_id: u8,
    pub sequence: u64,
    pub ack_bits: u64,
    pub timestamp_micros: i128,
}

impl PacketHeader {
    pub fn new(channel_id: u8, sequence: u64, ack_bits: u64) -> Self {
        Self {
            channel_id,
            sequence,
            ack_bits,
            timestamp_micros: OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000,
        }
    }
}

/// Header für fragmentierte Nachrichten.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct FragmentHeader {
    pub sequence: u64,
    pub fragment_index: u16,
    pub fragment_count: u16,
}

/// Ein paketiertes Payload inklusive Header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEnvelope {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl PacketEnvelope {
    pub fn new(header: PacketHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }
}
