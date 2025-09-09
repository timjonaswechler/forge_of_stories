//! Protocol Frame Definitions (Forge of Stories – Network Server)
//!
//! Scope (Milestone M1):
//! - Minimal set of frames required for initial QUIC handshake & liveness (Ping/Pong).
//! - Handshake negotiation (ClientHello / ServerHello) + failure reporting (HandshakeError).
//! - Transport-level utility frames (Ping / Pong).
//!
//! Encoding / Framing:
//! - These logical frames are wrapped by a length-prefix + binary codec (see `codec.rs`).
//! - Actual on-the-wire representation depends on the active feature (e.g. `proto_bincode`).
//!
//! Separation of Concerns:
//! - This module only declares data structures (no I/O logic).
//! - Handshake state machine lives in `handshake.rs`.
//! - Session registry in `session.rs`.
//!
//! Versioning Strategy:
//! - `ClientHello.version` is compared against `SUPPORTED_VERSIONS` in the handshake layer.
//! - Future protocol evolution can add new variants (keep them non-breaking by adding enums
//!   only at the end and using serde `deny_unknown_fields` selectively if needed).
//!
//! Security / Future:
//! - `token` (ClientHello) will later carry an auth / anti-abuse token (e.g. HMAC).
//! - Rate limiting / replay protection handled outside pure frame definitions.
//!
//! Logging Targets (suggested):
//! - server::net::frames (encode/decode events, only enable at trace/debug)
//!
//! (C) Forge of Stories

use serde::{Deserialize, Serialize};

/// Error categories for the handshake phase.
/// Keep codes stable for client-side mapping.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum HandshakeErrorCode {
    UnsupportedVersion = 1,
    AuthFailed = 2,
    RateLimited = 3,
    Malformed = 4,
    Internal = 5,
    Timeout = 6,
}

impl HandshakeErrorCode {
    /// Human readable short label (stable).
    pub fn label(self) -> &'static str {
        match self {
            Self::UnsupportedVersion => "unsupported_version",
            Self::AuthFailed => "auth_failed",
            Self::RateLimited => "rate_limited",
            Self::Malformed => "malformed",
            Self::Internal => "internal",
            Self::Timeout => "timeout",
        }
    }
}

/// Frames exchanged strictly before session establishment (and for reporting failures).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeFrame {
    /// First packet from the client.
    ClientHello {
        /// Desired / supported protocol version of the client.
        version: u16,
        /// Optional authentication / admission token (opaque to server until validated).
        token: Option<Vec<u8>>,
    },
    /// Positive server response after validating `ClientHello`.
    ServerHello {
        /// Assigned session identifier (unique during server lifetime or epoch).
        session_id: u64,
        /// Version actually accepted (chosen from client's offer).
        accepted_version: u16,
    },
    /// Terminal handshake failure – after sending this server will disconnect.
    HandshakeError {
        code: HandshakeErrorCode,
        /// Short textual diagnostic (NOT intended for end-users in production).
        message: String,
    },
}

/// Non-handshake control frames (post-session or generic transport utility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportFrame {
    /// Liveness probe – echoes back as Pong with same value.
    Ping(u64),
    /// Response to Ping – same opaque correlation value.
    Pong(u64),
    // Future: Disconnect(reason_code), FlowControl, FragmentedData, etc.
}

/// Envelope for all protocol-level frames.
/// Additional categories (e.g., Gameplay / WorldSync) would appear as new enum variants or
/// via nested higher-level framing once sessions are established.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frame {
    Handshake(HandshakeFrame),
    Transport(TransportFrame),
    // Future: Session(SessionFrame), ReliableData(...), UnreliableData(...)
}

impl Frame {
    /// Returns true if this frame is a client handshake initiation.
    pub fn is_client_hello(&self) -> bool {
        matches!(self, Frame::Handshake(HandshakeFrame::ClientHello { .. }))
    }

    /// Convenience constructor for a handshake error frame.
    pub fn handshake_error(code: HandshakeErrorCode, msg: impl Into<String>) -> Self {
        Frame::Handshake(HandshakeFrame::HandshakeError {
            code,
            message: msg.into(),
        })
    }
}

// -------------------------------------------------------------------------------------------------
// Tests (unit – serialization roundtrips)
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "proto_bincode")]
    #[test]
    fn roundtrip_client_hello() {
        let f = Frame::Handshake(HandshakeFrame::ClientHello {
            version: 1,
            token: Some(vec![1, 2, 3]),
        });
        let bin = bincode::serialize(&f).expect("serialize");
        let de: Frame = bincode::deserialize(&bin).expect("deserialize");
        match de {
            Frame::Handshake(HandshakeFrame::ClientHello { version, token }) => {
                assert_eq!(version, 1);
                assert_eq!(token, Some(vec![1, 2, 3]));
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[cfg(feature = "proto_bincode")]
    #[test]
    fn roundtrip_server_hello() {
        let f = Frame::Handshake(HandshakeFrame::ServerHello {
            session_id: 42,
            accepted_version: 1,
        });
        let bin = bincode::serialize(&f).expect("serialize");
        let de: Frame = bincode::deserialize(&bin).expect("deserialize");
        match de {
            Frame::Handshake(HandshakeFrame::ServerHello {
                session_id,
                accepted_version,
            }) => {
                assert_eq!(session_id, 42);
                assert_eq!(accepted_version, 1);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[cfg(feature = "proto_bincode")]
    #[test]
    fn handshake_error_helper() {
        let f = Frame::handshake_error(HandshakeErrorCode::Malformed, "bad");
        let bin = bincode::serialize(&f).unwrap();
        let de: Frame = bincode::deserialize(&bin).unwrap();
        match de {
            Frame::Handshake(HandshakeFrame::HandshakeError { code, message }) => {
                assert_eq!(code, HandshakeErrorCode::Malformed);
                assert_eq!(message, "bad");
            }
            _ => panic!("unexpected variant"),
        }
    }

    #[test]
    fn labels_unique() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for c in [
            HandshakeErrorCode::UnsupportedVersion,
            HandshakeErrorCode::AuthFailed,
            HandshakeErrorCode::RateLimited,
            HandshakeErrorCode::Malformed,
            HandshakeErrorCode::Internal,
            HandshakeErrorCode::Timeout,
        ] {
            assert!(set.insert(c.label()), "duplicate label {}", c.label());
        }
    }
}
