use anyhow::Result;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::codec::{DEFAULT_MAX_FRAME_SIZE, recv_frame_bincode, send_frame_bincode};

/// Extension trait for bincode-serializing messages onto a QUIC send stream.
#[allow(async_fn_in_trait)]
pub trait QuinnSendBincodeExt {
    /// Serialize `value` with bincode and write a length-prefixed frame to this stream.
    async fn send_bincode<T: Serialize>(&mut self, value: &T) -> Result<()>;
}

impl QuinnSendBincodeExt for quinn::SendStream {
    #[inline]
    async fn send_bincode<T: Serialize>(&mut self, value: &T) -> Result<()> {
        send_frame_bincode(self, value).await
    }
}

/// Extension trait for reading bincode-serialized messages from a QUIC recv stream.
#[allow(async_fn_in_trait)]
pub trait QuinnRecvBincodeExt {
    /// Read a single length-prefixed frame and deserialize it with bincode.
    ///
    /// Uses `DEFAULT_MAX_FRAME_SIZE` as an upper bound to guard allocations.
    async fn recv_bincode<T: DeserializeOwned>(&mut self) -> Result<T>;

    /// Read a single length-prefixed frame and deserialize it with bincode,
    /// enforcing the provided `max_frame_size`.
    async fn recv_bincode_with_limit<T: DeserializeOwned>(
        &mut self,
        max_frame_size: usize,
    ) -> Result<T>;
}

impl QuinnRecvBincodeExt for quinn::RecvStream {
    #[inline]
    async fn recv_bincode<T: DeserializeOwned>(&mut self) -> Result<T> {
        recv_frame_bincode(self, DEFAULT_MAX_FRAME_SIZE).await
    }

    #[inline]
    async fn recv_bincode_with_limit<T: DeserializeOwned>(
        &mut self,
        max_frame_size: usize,
    ) -> Result<T> {
        recv_frame_bincode(self, max_frame_size).await
    }
}

/// Convenience helpers around `quinn::Connection` for opening streams.

/// Open a bidirectional QUIC stream on the provided connection.
///
/// This is a thin wrapper around `quinn::Connection::open_bi` to keep the import surface
/// small wherever only shared messaging helpers are used.
#[inline]
pub async fn open_bi(conn: &quinn::Connection) -> Result<(quinn::SendStream, quinn::RecvStream)> {
    let (send, recv) = conn.open_bi().await?;
    Ok((send, recv))
}

/// Open a unidirectional QUIC send stream on the provided connection.
///
/// Thin wrapper around `quinn::Connection::open_uni`.
#[inline]
pub async fn open_uni(conn: &quinn::Connection) -> Result<quinn::SendStream> {
    let send = conn.open_uni().await?;
    Ok(send)
}
