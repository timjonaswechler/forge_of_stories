//! Serialisierungs- und Deserialisierungshilfen.

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

/// Trait für serialisierbare Nachrichten-Payloads.
pub trait MessageSerializer: Send + Sync + 'static {
    fn serialize<T>(&self, value: &T) -> Result<Vec<u8>, SerializationError>
    where
        T: Serialize;

    fn deserialize<T>(&self, bytes: &[u8]) -> Result<T, SerializationError>
    where
        T: DeserializeOwned;
}

/// Standard-Implementierung basierend auf `bincode`.
#[derive(Debug, Default)]
pub struct BincodeSerializer;

impl MessageSerializer for BincodeSerializer {
    fn serialize<T>(&self, value: &T) -> Result<Vec<u8>, SerializationError>
    where
        T: Serialize,
    {
        bincode::serde::encode_to_vec(value, bincode::config::standard())
            .map_err(SerializationError::BincodeEncode)
    }

    fn deserialize<T>(&self, bytes: &[u8]) -> Result<T, SerializationError>
    where
        T: DeserializeOwned,
    {
        let (value, _len) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .map_err(SerializationError::BincodeDecode)?;
        Ok(value)
    }
}

/// Fehler, die bei (De-)Serialisierung auftreten können.
#[derive(Debug, Error)]
pub enum SerializationError {
    #[error("bincode encode error: {0}")]
    BincodeEncode(bincode::error::EncodeError),
    #[error("bincode decode error: {0}")]
    BincodeDecode(bincode::error::DecodeError),
}
