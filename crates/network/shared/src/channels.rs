//! Definition der logischen Netzwerkkanäle und ihrer Eigenschaften.

use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};

/// Art des Kanals, bestimmt Zuverlässigkeit und Reihenfolge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelKind {
    ReliableOrdered,
    ReliableUnordered,
    UnreliableSequenced,
    Control,
}

/// Beschreibt einen einzelnen Kanal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDescriptor {
    pub id: u8,
    pub kind: ChannelKind,
    pub label: Cow<'static, str>,
    pub priority: u8,
    pub datagram: bool,
}

impl ChannelDescriptor {
    pub const fn new(
        id: u8,
        kind: ChannelKind,
        label: Cow<'static, str>,
        priority: u8,
        datagram: bool,
    ) -> Self {
        Self {
            id,
            kind,
            label,
            priority,
            datagram,
        }
    }
}

/// Registry aller bekannten Kanäle.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelRegistry {
    by_id: HashMap<u8, ChannelDescriptor>,
}

impl ChannelRegistry {
    pub fn new(channels: impl IntoIterator<Item = ChannelDescriptor>) -> Self {
        let mut registry = HashMap::new();
        for descriptor in channels {
            registry.insert(descriptor.id, descriptor);
        }
        Self { by_id: registry }
    }

    pub fn descriptor(&self, id: u8) -> Option<&ChannelDescriptor> {
        self.by_id.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChannelDescriptor> {
        self.by_id.values()
    }
}
