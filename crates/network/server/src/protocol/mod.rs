#![allow(clippy::module_inception)]
//! Protocol Namespace (Forge of Stories – Network Server)
//!
//! This namespace groups all protocol-layer building blocks that sit
//! directly above the raw transport providers.
//!
//! Current Milestone (M1) Contents:
//! - `frames`    : Logical handshake & transport control frames (serialization friendly).
//! - `codec`     : Length‑prefixed binary framing + (optional) bincode serialization.
//! - `handshake` : State machine for promoting raw connections to live sessions.
//! - `metrics`   : Lightweight counters & periodic logging for network events.
//!
//! Responsibilities:
//! - Decouple transport event ingestion (byte streams) from higher-level
//!   session / gameplay logic.
//! - Provide a stable evolution path (new frames or categories can be
//!   added without breaking existing clients, given version negotiation).
//!
//! Not Here (yet):
//! - Gameplay / world replication frames
//! - Fragmentation / large payload strategies
//! - Reliability beyond QUIC semantics
//! - Encryption (handled by TLS / QUIC) or compression
//!
//! Logging Targets (Suggested):
//! - server::net::frames
//! - server::net::handshake
//! - server::net::session
//! - server::net::metrics
//!
//! Feature Flags:
//! - `proto_bincode`: Enables binary encode/decode via `bincode`; without it
//!   only debug encode placeholder exists (decode disabled).
//!
//! Public Prelude:
//! Import `protocol::prelude::*` for the most commonly required types
//! (Frame, HandshakeFrame, TransportFrame, HandshakeErrorCode, FrameCodec,
//! handshake events).
//!
//! (C) Forge of Stories

pub mod codec;
pub mod frames;
pub mod handshake;
pub mod metrics;

/// Convenient re-exports for external modules wanting to interact with the
/// protocol layer without deep path drilling.
pub mod prelude {
    pub use super::codec::{FrameCodec, FrameDecoder};
    pub use super::frames::{Frame, HandshakeErrorCode, HandshakeFrame, TransportFrame};
    pub use super::handshake::{
        NetSessionClosed, NetSessionEstablished, PendingHandshakes, SUPPORTED_VERSIONS,
    };
    pub use super::metrics::NetMetrics;
}
