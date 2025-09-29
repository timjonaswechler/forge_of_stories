//! Streng typisierte Bezeichner für Sessions, Clients und Transports.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// Generischer Generator für inkrementelle IDs.
#[derive(Debug, Clone)]
pub struct IdGenerator {
    counter: Arc<AtomicU64>,
}

impl IdGenerator {
    pub fn new(start: u64) -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(start)),
        }
    }

    #[inline]
    pub fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(0)
            }
        }
    };
}

id_type!(ClientId);
id_type!(ServerId);
id_type!(SessionId);

/// Kombinierter Schlüssel für Transport-spezifische Zuordnungen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerKey {
    pub session: SessionId,
    pub client: ClientId,
}

impl PeerKey {
    pub const fn new(session: SessionId, client: ClientId) -> Self {
        Self { session, client }
    }
}
