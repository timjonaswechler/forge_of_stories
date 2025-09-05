use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Default maximum accepted frame size (1 MiB).
/// Use a custom limit by passing your own `max_frame_size` to `recv_frame_bincode`.
pub const DEFAULT_MAX_FRAME_SIZE: usize = 1 * 1024 * 1024;

/// Send a length-prefixed (u32, big-endian) frame with a bincode-serialized payload.
///
/// Layout:
/// - 4 bytes: payload length (u32, big-endian)
/// - N bytes: payload
///
/// Returns when all bytes have been written to the underlying writer.
pub async fn send_frame_bincode<W, T>(writer: &mut W, value: &T) -> Result<()>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let payload = bincode::serialize(value).context("bincode serialize failed")?;
    // Using u32 for the length prefix; enforce the limit here
    if payload.len() > u32::MAX as usize {
        bail!(
            "frame too large for u32 length prefix: {} bytes",
            payload.len()
        );
    }

    let len = (payload.len() as u32).to_be_bytes();
    writer
        .write_all(&len)
        .await
        .context("failed to write frame length")?;
    writer
        .write_all(&payload)
        .await
        .context("failed to write frame payload")?;
    writer.flush().await.context("failed to flush writer")?;
    Ok(())
}

/// Receive a length-prefixed (u32, big-endian) frame and deserialize its payload with bincode.
///
/// `max_frame_size` guards against malicious or accidental large allocations.
/// If the incoming frame length exceeds `max_frame_size`, this returns an error.
pub async fn recv_frame_bincode<R, T>(reader: &mut R, max_frame_size: usize) -> Result<T>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let payload = recv_frame_bytes(reader, max_frame_size).await?;
    let value =
        bincode::deserialize::<T>(&payload).context("bincode deserialize failed for frame")?;
    Ok(value)
}

/// Low-level helper: send a raw length-prefixed (u32, big-endian) frame.
pub async fn send_frame_bytes<W>(writer: &mut W, payload: &[u8]) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    if payload.len() > u32::MAX as usize {
        bail!(
            "frame too large for u32 length prefix: {} bytes",
            payload.len()
        );
    }

    let len = (payload.len() as u32).to_be_bytes();
    writer
        .write_all(&len)
        .await
        .context("failed to write frame length")?;
    writer
        .write_all(payload)
        .await
        .context("failed to write frame payload")?;
    writer.flush().await.context("failed to flush writer")?;
    Ok(())
}

/// Low-level helper: receive a raw length-prefixed (u32, big-endian) frame.
pub async fn recv_frame_bytes<R>(reader: &mut R, max_frame_size: usize) -> Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    let mut len_buf = [0u8; 4];
    reader
        .read_exact(&mut len_buf)
        .await
        .context("failed to read frame length")?;

    let len = u32::from_be_bytes(len_buf) as usize;
    if len > max_frame_size {
        bail!(
            "incoming frame length {} exceeds max_frame_size {}",
            len,
            max_frame_size
        );
    }

    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .await
        .context("failed to read frame payload")?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tokio::io::duplex;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct TestMsg {
        id: u32,
        name: String,
    }

    #[tokio::test]
    async fn round_trip_bincode() -> Result<()> {
        let (mut client, mut server) = duplex(64 * 1024);

        let msg = TestMsg {
            id: 42,
            name: "Aether".into(),
        };

        let send_task = tokio::spawn(async move {
            send_frame_bincode(&mut client, &msg).await.unwrap();
        });

        let recv_task = tokio::spawn(async move {
            let received: TestMsg = recv_frame_bincode(&mut server, DEFAULT_MAX_FRAME_SIZE)
                .await
                .unwrap();
            received
        });

        send_task.await.unwrap();
        let got = recv_task.await.unwrap();
        assert_eq!(got, msg);

        Ok(())
    }

    #[tokio::test]
    async fn rejects_too_large_frame() -> Result<()> {
        let (mut client, mut server) = duplex(64 * 1024);

        let big = vec![0u8; DEFAULT_MAX_FRAME_SIZE + 1];
        let send_task = tokio::spawn(async move {
            send_frame_bytes(&mut client, &big).await.unwrap();
        });

        let recv_task = tokio::spawn(async move {
            // Expect an error because the frame is larger than the limit.
            let res = recv_frame_bytes(&mut server, DEFAULT_MAX_FRAME_SIZE).await;
            res
        });

        send_task.await.unwrap();
        let res = recv_task.await.unwrap();
        assert!(res.is_err());
        Ok(())
    }
}
