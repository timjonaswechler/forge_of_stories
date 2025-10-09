//! Loopback transport for in-memory client-server communication.
//!
//! Provides a transport implementation that keeps client and server in the same
//! process without touching the network stack. This is primarily used for
//! singleplayer runs and local testing.

use std::{
    any::Any,
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use tracing::warn;
use uuid::uuid;

use crate::{
    ClientEvent, ClientId, DisconnectReason, TransportCapabilities, TransportError, TransportEvent,
    channels::ChannelId,
};

use super::{ClientTransport, ServerTransport, TransportPayload, TransportResult};

/// Loopback transport capabilities.
///
/// Loopback transports support reliable streams and datagrams, but not
/// unreliable streams (communication is always reliable in-memory).
const LOOPBACK_CAPABILITIES: TransportCapabilities = TransportCapabilities {
    supports_reliable_streams: true,
    supports_unreliable_streams: false,
    supports_datagrams: true,
    max_channels: 255,
};

/// Default client id used by the loopback transport.
/// Matches historical behaviour where the embedded host used a fixed UUID.
const LOOPBACK_CLIENT_ID: ClientId = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

/// Error type for loopback transport operations.
#[derive(Debug, thiserror::Error)]
pub enum LoopbackError {
    #[error("loopback transport not connected")]
    NotConnected,
    #[error("loopback transport already connected")]
    AlreadyConnected,
    #[error("loopback transport invalid client {0}")]
    InvalidClient(ClientId),
}

impl From<LoopbackError> for TransportError {
    fn from(err: LoopbackError) -> Self {
        match err {
            LoopbackError::NotConnected => TransportError::NotReady,
            LoopbackError::AlreadyConnected => {
                TransportError::Other("loopback transport already connected".into())
            }
            LoopbackError::InvalidClient(id) => {
                TransportError::Other(format!("loopback transport invalid client {id}"))
            }
        }
    }
}

#[derive(Debug, Default)]
struct DirectMessageQueue {
    inner: VecDeque<DirectMessage>,
}

impl DirectMessageQueue {
    fn push(&mut self, message: DirectMessage) {
        self.inner.push_back(message);
    }

    fn drain(&mut self) -> VecDeque<DirectMessage> {
        std::mem::take(&mut self.inner)
    }
}

struct DirectMessage {
    channel: ChannelId,
    payload: Box<dyn Any + Send>,
}

impl std::fmt::Debug for DirectMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectMessage")
            .field("channel", &self.channel)
            .field("payload_type", &self.payload.type_id())
            .finish()
    }
}

#[derive(Debug)]
struct SharedLoopbackState {
    connected: AtomicBool,
    client_id: ClientId,
    server_events: Mutex<VecDeque<TransportEvent>>,
    client_events: Mutex<VecDeque<ClientEvent>>,
    client_to_server: Mutex<VecDeque<TransportPayload>>,
    server_to_client: Mutex<VecDeque<TransportPayload>>,
    server_direct_messages: Mutex<DirectMessageQueue>,
}

impl SharedLoopbackState {
    fn new() -> Self {
        Self {
            connected: AtomicBool::new(false),
            client_id: LOOPBACK_CLIENT_ID,
            server_events: Mutex::new(VecDeque::new()),
            client_events: Mutex::new(VecDeque::new()),
            client_to_server: Mutex::new(VecDeque::new()),
            server_to_client: Mutex::new(VecDeque::new()),
            server_direct_messages: Mutex::new(DirectMessageQueue::default()),
        }
    }

    fn push_server_event(&self, event: TransportEvent) {
        if let Ok(mut queue) = self.server_events.lock() {
            queue.push_back(event);
        }
    }

    fn push_client_event(&self, event: ClientEvent) {
        if let Ok(mut queue) = self.client_events.lock() {
            queue.push_back(event);
        }
    }

    fn push_server_direct_message(&self, channel: ChannelId, payload: Box<dyn Any + Send>) {
        if let Ok(mut queue) = self.server_direct_messages.lock() {
            queue.push(DirectMessage { channel, payload });
        }
    }

    fn drain_server_direct_messages(&self) -> VecDeque<DirectMessage> {
        if let Ok(mut queue) = self.server_direct_messages.lock() {
            return queue.drain();
        }

        VecDeque::new()
    }
}

/// A pair of connected loopback transports (client and server halves).
pub struct LoopbackPair {
    pub client: LoopbackClientTransport,
    pub server: LoopbackServerTransport,
}

impl LoopbackPair {
    /// Creates a new loopback transport pair with shared state.
    pub fn new() -> Self {
        let state = Arc::new(SharedLoopbackState::new());
        Self {
            client: LoopbackClientTransport::new(Arc::clone(&state)),
            server: LoopbackServerTransport::new(state),
        }
    }
}

impl Default for LoopbackPair {
    fn default() -> Self {
        Self::new()
    }
}

/// Client-side loopback transport implementation.
pub struct LoopbackClientTransport {
    state: Arc<SharedLoopbackState>,
}

impl LoopbackClientTransport {
    fn new(state: Arc<SharedLoopbackState>) -> Self {
        Self { state }
    }

    fn ensure_connected(&self) -> Result<(), LoopbackError> {
        if self.state.connected.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(LoopbackError::NotConnected)
        }
    }

    /// Returns the fixed capabilities for the loopback client.
    pub fn capabilities(&self) -> TransportCapabilities {
        LOOPBACK_CAPABILITIES
    }

    /// Drains typed direct messages sent by the server without serialization.
    ///
    /// Each entry contains the logical channel alongside the typed payload.
    /// Messages of a different type are dropped with a warning.
    pub fn poll_direct<M>(&mut self, output: &mut Vec<(ChannelId, M)>) -> TransportResult<()>
    where
        M: Send + 'static,
    {
        self.ensure_connected()?;

        let mut drained = self.state.drain_server_direct_messages();
        while let Some(packet) = drained.pop_front() {
            match packet.payload.downcast::<M>() {
                Ok(message) => output.push((packet.channel, *message)),
                Err(untyped) => {
                    warn!(
                        "loopback direct message dropped: expected type {}, got {:?}",
                        std::any::type_name::<M>(),
                        untyped.type_id()
                    );
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for LoopbackClientTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopbackClientTransport")
            .field("connected", &self.state.connected.load(Ordering::SeqCst))
            .finish()
    }
}

impl ClientTransport for LoopbackClientTransport {
    type ConnectTarget = ();

    fn poll_events(&mut self, output: &mut Vec<ClientEvent>) {
        if let Ok(mut queue) = self.state.client_events.lock() {
            output.extend(queue.drain(..));
        }

        if let Ok(mut queue) = self.state.server_to_client.lock() {
            while let Some(payload) = queue.pop_front() {
                match payload {
                    TransportPayload::Message { channel, payload } => {
                        output.push(ClientEvent::Message { channel, payload })
                    }
                    TransportPayload::Datagram { payload } => {
                        output.push(ClientEvent::Datagram { payload })
                    }
                }
            }
        }
    }

    fn connect(&mut self, _: Self::ConnectTarget) -> TransportResult<()> {
        if self
            .state
            .connected
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(LoopbackError::AlreadyConnected.into());
        }

        self.state
            .push_client_event(ClientEvent::Connected { client_id: None });
        self.state.push_server_event(TransportEvent::PeerConnected {
            client: self.state.client_id,
        });

        Ok(())
    }

    fn disconnect(&mut self) -> TransportResult<()> {
        self.ensure_connected()?;
        self.state.connected.store(false, Ordering::SeqCst);

        self.state.push_client_event(ClientEvent::Disconnected {
            reason: DisconnectReason::Graceful,
        });
        self.state
            .push_server_event(TransportEvent::PeerDisconnected {
                client: self.state.client_id,
                reason: DisconnectReason::Graceful,
            });
        Ok(())
    }

    fn send(&mut self, payload: TransportPayload) -> TransportResult<()> {
        self.ensure_connected()?;
        if let Ok(mut queue) = self.state.client_to_server.lock() {
            queue.push_back(payload);
        }
        Ok(())
    }
}

/// Server-side loopback transport implementation.
pub struct LoopbackServerTransport {
    state: Arc<SharedLoopbackState>,
}

impl LoopbackServerTransport {
    fn new(state: Arc<SharedLoopbackState>) -> Self {
        Self { state }
    }

    fn ensure_connected(&self) -> Result<(), LoopbackError> {
        if self.state.connected.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(LoopbackError::NotConnected)
        }
    }

    /// Returns the fixed capabilities for the loopback server.
    pub fn capabilities(&self) -> TransportCapabilities {
        LOOPBACK_CAPABILITIES
    }

    /// Returns the client identifier associated with the loopback connection.
    pub fn client_id(&self) -> ClientId {
        self.state.client_id
    }

    /// Forces the loopback client to disconnect with the provided reason.
    pub fn force_disconnect(&mut self, reason: DisconnectReason) -> TransportResult<()> {
        if self
            .state
            .connected
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(LoopbackError::NotConnected.into());
        }

        self.state
            .push_client_event(ClientEvent::Disconnected { reason });
        self.state
            .push_server_event(TransportEvent::PeerDisconnected {
                client: self.state.client_id,
                reason,
            });
        Ok(())
    }

    /// Sends a typed payload directly to the loopback client without serialization.
    ///
    /// This is primarily used for singleplayer fast paths where server and client
    /// share memory. Messages are stored internally and can be drained via
    /// [`LoopbackClientTransport::poll_direct`].
    pub fn send_direct<M>(&mut self, channel: ChannelId, message: M) -> TransportResult<()>
    where
        M: Send + 'static,
    {
        self.ensure_connected()?;
        self.state
            .push_server_direct_message(channel, Box::new(message));
        Ok(())
    }
}

impl std::fmt::Debug for LoopbackServerTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopbackServerTransport")
            .field("client_id", &self.state.client_id)
            .field("connected", &self.state.connected.load(Ordering::SeqCst))
            .finish()
    }
}

impl ServerTransport for LoopbackServerTransport {
    fn poll_events(&mut self, output: &mut Vec<TransportEvent>) {
        if let Ok(mut queue) = self.state.server_events.lock() {
            output.extend(queue.drain(..));
        }

        if let Ok(mut queue) = self.state.client_to_server.lock() {
            while let Some(payload) = queue.pop_front() {
                match payload {
                    TransportPayload::Message { channel, payload } => {
                        output.push(TransportEvent::Message {
                            client: self.state.client_id,
                            channel,
                            payload,
                        })
                    }
                    TransportPayload::Datagram { payload } => {
                        output.push(TransportEvent::Datagram {
                            client: self.state.client_id,
                            payload,
                        })
                    }
                }
            }
        }
    }

    fn send(&mut self, client: ClientId, payload: TransportPayload) -> TransportResult<()> {
        if client != self.state.client_id {
            return Err(LoopbackError::InvalidClient(client).into());
        }
        self.ensure_connected()?;
        if let Ok(mut queue) = self.state.server_to_client.lock() {
            queue.push_back(payload);
        }
        Ok(())
    }

    fn broadcast(&mut self, payload: TransportPayload) -> TransportResult<()> {
        if self.state.connected.load(Ordering::SeqCst) {
            if let Ok(mut queue) = self.state.server_to_client.lock() {
                queue.push_back(payload);
            }
        }
        Ok(())
    }

    fn broadcast_excluding(
        &mut self,
        exclude: &[ClientId],
        payload: TransportPayload,
    ) -> TransportResult<()> {
        if exclude.iter().any(|id| *id == self.state.client_id) {
            return Ok(());
        }
        self.broadcast(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn connected_pair() -> LoopbackPair {
        let mut pair = LoopbackPair::new();
        pair.client.connect(()).unwrap();
        // Drain connection events
        let mut client_events = Vec::new();
        pair.client.poll_events(&mut client_events);
        let mut server_events = Vec::new();
        pair.server.poll_events(&mut server_events);
        LoopbackPair {
            client: pair.client,
            server: pair.server,
        }
    }

    #[test]
    fn test_loopback_connect_events() {
        let mut pair = LoopbackPair::new();
        pair.client.connect(()).unwrap();

        let mut client_events = Vec::new();
        pair.client.poll_events(&mut client_events);
        assert!(matches!(
            client_events.as_slice(),
            [ClientEvent::Connected { client_id: None }]
        ));

        let mut server_events = Vec::new();
        pair.server.poll_events(&mut server_events);
        assert!(matches!(
            server_events.as_slice(),
            [TransportEvent::PeerConnected { client }] if *client == LOOPBACK_CLIENT_ID
        ));
    }

    #[test]
    fn test_loopback_send_roundtrip() {
        let mut pair = connected_pair();

        pair.client
            .send(TransportPayload::message(0, Bytes::from("Client Hello")))
            .unwrap();

        let mut server_events = Vec::new();
        pair.server.poll_events(&mut server_events);
        assert!(matches!(
            server_events.as_slice(),
            [TransportEvent::Message { client, channel, payload }]
            if *client == LOOPBACK_CLIENT_ID && *channel == 0 && payload.as_ref() == b"Client Hello"
        ));

        pair.server
            .send(
                LOOPBACK_CLIENT_ID,
                TransportPayload::message(0, Bytes::from("Server Response")),
            )
            .unwrap();

        let mut client_events = Vec::new();
        pair.client.poll_events(&mut client_events);
        assert!(matches!(
            client_events.as_slice(),
            [ClientEvent::Message { channel, payload }]
            if *channel == 0 && payload.as_ref() == b"Server Response"
        ));
    }

    #[test]
    fn test_loopback_broadcast() {
        let mut pair = connected_pair();

        pair.server
            .broadcast(TransportPayload::message(1, Bytes::from("Broadcast")))
            .unwrap();

        let mut client_events = Vec::new();
        pair.client.poll_events(&mut client_events);
        assert!(matches!(
            client_events.as_slice(),
            [ClientEvent::Message { channel, payload }]
            if *channel == 1 && payload.as_ref() == b"Broadcast"
        ));
    }

    #[test]
    fn test_loopback_disconnect_lifecycle() {
        let mut pair = connected_pair();

        pair.client.disconnect().unwrap();

        let mut client_events = Vec::new();
        pair.client.poll_events(&mut client_events);
        assert!(matches!(
            client_events.as_slice(),
            [ClientEvent::Disconnected {
                reason: DisconnectReason::Graceful
            }]
        ));

        let mut server_events = Vec::new();
        pair.server.poll_events(&mut server_events);
        assert!(matches!(
            server_events.as_slice(),
            [TransportEvent::PeerDisconnected { client, reason: DisconnectReason::Graceful }]
            if *client == LOOPBACK_CLIENT_ID
        ));
    }

    #[test]
    fn test_force_disconnect() {
        let mut pair = connected_pair();

        pair.server
            .force_disconnect(DisconnectReason::Kicked)
            .unwrap();

        let mut client_events = Vec::new();
        pair.client.poll_events(&mut client_events);
        assert!(matches!(
            client_events.as_slice(),
            [ClientEvent::Disconnected {
                reason: DisconnectReason::Kicked
            }]
        ));

        let mut server_events = Vec::new();
        pair.server.poll_events(&mut server_events);
        assert!(matches!(
            server_events.as_slice(),
            [TransportEvent::PeerDisconnected { client, reason: DisconnectReason::Kicked }]
            if *client == LOOPBACK_CLIENT_ID
        ));
    }

    #[test]
    fn test_capabilities() {
        let pair = LoopbackPair::new();
        assert_eq!(pair.client.capabilities(), LOOPBACK_CAPABILITIES);
        assert_eq!(pair.server.capabilities(), LOOPBACK_CAPABILITIES);
    }

    #[test]
    fn test_loopback_direct_message_roundtrip() {
        let mut pair = connected_pair();

        #[derive(Debug, PartialEq, Eq)]
        struct DummyMessage(u32);

        pair.server.send_direct(0, DummyMessage(42)).unwrap();

        let mut direct_messages = Vec::new();
        pair.client
            .poll_direct::<DummyMessage>(&mut direct_messages)
            .unwrap();

        assert_eq!(direct_messages.len(), 1);
        let (channel, payload) = direct_messages.into_iter().next().unwrap();
        assert_eq!(channel, 0);
        assert_eq!(payload, DummyMessage(42));

        // second poll returns nothing
        let mut empty = Vec::new();
        pair.client.poll_direct::<DummyMessage>(&mut empty).unwrap();
        assert!(empty.is_empty());
    }
}
