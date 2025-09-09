//! Frame Codec (Forge of Stories – Network Server)
//!
//! Responsibilities:
//! - Length-prefix framing: [u32_be length][payload bytes]
//! - (De)serialization of `Frame` (see `frames.rs`) via `bincode` when the
//!   `proto_bincode` feature is enabled.
//! - Incremental decode support (stream oriented); caller feeds arbitrary
//!   raw byte chunks (e.g. from QUIC stream) and pulls zero or more frames.
//!
//! Non-Goals (for M1):
//! - Zero-copy optimization (acceptable to copy payload into an owned Vec).
//! - Compression / encryption (handled at QUIC / TLS layer).
//! - Multi-frame batching heuristics (caller can simply loop).
//!
//! Error Handling:
//! - Oversized length prefix (> `max_frame_bytes`) => hard error, caller should
//!   disconnect the offending connection.
//! - Decode errors propagate as `anyhow::Error`; caller translates into
//!   protocol-level HandshakeError (Malformed) or disconnect.
//!
//! Feature Flags:
//! - `proto_bincode`: Enables actual binary (de)serialization using `bincode`.
//!   Without this feature, encoding falls back to a debug-string placeholder
//!   and decode is NOT supported (returns an error).
//!
//! Logging:
//! - Consider adding trace logs (disabled by default) around encode / decode
//!   if deep protocol debugging is needed (target: `server::net::frames`).
//!
//! Safety / Limits:
//! - `max_frame_bytes` is enforced before allocation for decode payload.
//!
//! (C) Forge of Stories – MIT / Apache-2 (as applicable)

// (removed unused import: Cursor)

use anyhow::{Result, bail};

use super::frames::Frame;

/// Codec configuration / stateless helper.
#[derive(Debug, Clone)]
pub struct FrameCodec {
    /// Maximum allowed serialized frame payload size (bytes), excluding the 4-byte length prefix.
    pub max_frame_bytes: u32,
}

impl FrameCodec {
    pub fn new(max_frame_bytes: u32) -> Self {
        Self { max_frame_bytes }
    }

    /// Encode a single `Frame` and append to `out`.
    ///
    /// Layout: [len: u32 BE][payload bytes...]
    pub fn encode(&self, frame: &Frame, out: &mut Vec<u8>) -> Result<()> {
        let start = out.len();
        // Reserve 4 bytes for length prefix
        out.extend_from_slice(&[0, 0, 0, 0]);

        #[cfg(feature = "proto_bincode")]
        {
            bincode::serialize_into(out, frame)?;
        }

        #[cfg(not(feature = "proto_bincode"))]
        {
            // Placeholder debug representation – not meant for production, no decoding support.
            let dbg_str = format!("{frame:?}");
            out.extend_from_slice(dbg_str.as_bytes());
        }

        let payload_len = (out.len() - start - 4) as u32;
        if payload_len > self.max_frame_bytes {
            // Roll back appended bytes (leave previous data intact)
            out.truncate(start);
            bail!(
                "encoded frame exceeds max_frame_bytes ({} > {})",
                payload_len,
                self.max_frame_bytes
            );
        }
        out[start..start + 4].copy_from_slice(&payload_len.to_be_bytes());
        Ok(())
    }

    /// Attempt to decode exactly one frame from `buffer`.
    ///
    /// Returns:
    /// - Ok(Some(Frame)) if a full frame was decoded (and removed from buffer)
    /// - Ok(None) if not enough data yet
    /// - Err if malformed / violates size limit / decode error
    ///
    /// The buffer may contain additional bytes (subsequent frames) which remain untouched.
    pub fn try_decode(buffer: &mut Vec<u8>, max_frame_bytes: u32) -> Result<Option<Frame>> {
        if buffer.len() < 4 {
            return Ok(None);
        }
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&buffer[..4]);
        let frame_len = u32::from_be_bytes(len_bytes);

        if frame_len > max_frame_bytes {
            bail!("frame too large: {frame_len} > {max_frame_bytes}");
        }

        let total_needed = 4 + frame_len as usize;
        if buffer.len() < total_needed {
            return Ok(None);
        }

        #[allow(unused_variables)]
        let payload = buffer[4..total_needed].to_vec();
        // Drain consumed bytes
        buffer.drain(..total_needed);

        #[cfg(feature = "proto_bincode")]
        {
            let frame: Frame = bincode::deserialize(&payload)?;
            Ok(Some(frame))
        }

        #[cfg(not(feature = "proto_bincode"))]
        {
            // We cannot reconstruct from debug string – signal unsupported operation.
            bail!("decode unsupported without 'proto_bincode' feature");
        }
    }
}

/// Stateful incremental decoder.
/// Feed arbitrary chunks via `push_bytes`, then repeatedly call `next_frame`
/// until it returns Ok(None).
#[derive(Debug, Default)]
pub struct FrameDecoder {
    buf: Vec<u8>,
    max_frame_bytes: u32,
}

impl FrameDecoder {
    pub fn new(max_frame_bytes: u32) -> Self {
        Self {
            buf: Vec::new(),
            max_frame_bytes,
        }
    }

    /// Supply additional raw bytes.
    pub fn push_bytes(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    /// Attempt to pull one frame. See `FrameCodec::try_decode` semantics.
    pub fn next_frame(&mut self) -> Result<Option<Frame>> {
        FrameCodec::try_decode(&mut self.buf, self.max_frame_bytes)
    }

    /// Expose internal buffered (undecoded) byte count (for diagnostics).
    pub fn buffered_len(&self) -> usize {
        self.buf.len()
    }

    /// Clear buffer (e.g. after fatal error).
    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::super::frames::{HandshakeErrorCode, HandshakeFrame};
    use super::*;

    #[cfg(feature = "proto_bincode")]
    #[test]
    fn encode_decode_roundtrip() {
        let codec = FrameCodec::new(16 * 1024);
        let frame = Frame::Handshake(HandshakeFrame::ClientHello {
            version: 1,
            token: Some(vec![9, 8, 7]),
        });
        let mut buf = Vec::new();
        codec.encode(&frame, &mut buf).expect("encode");
        let mut aggregate = buf.clone();
        let decoded = FrameCodec::try_decode(&mut aggregate, 16 * 1024)
            .expect("decode result")
            .expect("some frame");
        match decoded {
            Frame::Handshake(HandshakeFrame::ClientHello { version, token }) => {
                assert_eq!(version, 1);
                assert_eq!(token, Some(vec![9, 8, 7]));
            }
            other => panic!("unexpected frame variant: {other:?}"),
        }
        assert!(aggregate.is_empty(), "buffer should be drained");
    }

    #[cfg(feature = "proto_bincode")]
    #[test]
    fn decoder_incremental() {
        let codec = FrameCodec::new(4096);
        let frame = Frame::Handshake(HandshakeFrame::HandshakeError {
            code: HandshakeErrorCode::Malformed,
            message: "oops".into(),
        });

        let mut serialized = Vec::new();
        codec.encode(&frame, &mut serialized).unwrap();

        // Feed in two parts
        let split = 5;
        let mut dec = FrameDecoder::new(4096);
        dec.push_bytes(&serialized[..split]);
        assert!(dec.next_frame().unwrap().is_none(), "should need more data");
        dec.push_bytes(&serialized[split..]);
        let f = dec.next_frame().unwrap().expect("frame now complete");
        match f {
            Frame::Handshake(HandshakeFrame::HandshakeError { code, message }) => {
                assert_eq!(code, HandshakeErrorCode::Malformed);
                assert_eq!(message, "oops");
            }
            _ => panic!("wrong variant"),
        }
        assert!(dec.next_frame().unwrap().is_none(), "no extra frame");
    }

    #[cfg(feature = "proto_bincode")]
    #[test]
    fn oversize_rejected() {
        let codec = FrameCodec::new(8);
        // Build an intentionally large message (payload > 8)
        let frame = Frame::Handshake(HandshakeFrame::ClientHello {
            version: 1,
            token: Some(vec![0; 32]),
        });
        let mut buf = Vec::new();
        let err = codec.encode(&frame, &mut buf).unwrap_err();
        assert!(
            err.to_string().contains("exceeds"),
            "unexpected error: {err}"
        );
    }

    #[cfg(feature = "proto_bincode")]
    #[test]
    fn oversize_decode_rejected() {
        // Manually craft length prefix larger than limit
        let mut buf = Vec::new();
        buf.extend_from_slice(&1000u32.to_be_bytes()); // length 1000
        buf.extend_from_slice(&vec![0u8; 1000]);

        let mut v = buf.clone();
        let err = FrameCodec::try_decode(&mut v, 64).unwrap_err();
        assert!(err.to_string().contains("frame too large"));
    }

    #[test]
    fn decoder_buffer_len() {
        let mut d = FrameDecoder::new(1024);
        d.push_bytes(&[1, 2, 3]);
        assert_eq!(d.buffered_len(), 3);
        d.clear();
        assert_eq!(d.buffered_len(), 0);
    }

    #[cfg(not(feature = "proto_bincode"))]
    #[test]
    fn decode_unsupported_without_feature() {
        let codec = FrameCodec::new(4096);
        let frame = Frame::Handshake(HandshakeFrame::ServerHello {
            session_id: 1,
            accepted_version: 1,
        });
        let mut out = Vec::new();
        codec.encode(&frame, &mut out).unwrap();
        let mut v = out;
        let err = FrameCodec::try_decode(&mut v, 4096).unwrap_err();
        assert!(err.to_string().contains("decode unsupported"));
    }
}
