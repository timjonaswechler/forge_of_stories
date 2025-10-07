//! Loopback transport for in-memory client-server communication.
//!
//! This transport enables singleplayer mode by providing zero-latency message passing
//! between client and server running in the same process, without any network overhead.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bytes::Bytes;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
    ClientEvent, ClientId, DisconnectReason, OutgoingMessage, TransportCapabilities, TransportEvent,
};

/// Error type for loopback transport operations.
#[derive(Debug, thiserror::Error)]
pub enum LoopbackError {
    #[error("Loopback channel closed")]
    ChannelClosed,
    #[error("Not connected")]
    NotConnected,
    #[error("Already connected")]
    AlreadyConnected,
}

/// A pair of connected loopback transports (client and server halves).
///
/// Creates bidirectional in-memory channels for zero-latency communication
/// between client and server running in the same process.
///
/// Note: This struct is not Clone because the underlying channels (UnboundedReceiver)
/// cannot be cloned. You must use Arc or other sharing mechanisms if needed.
pub struct LoopbackPair {
    pub client: LoopbackClientTransport,
    pub server: LoopbackServerTransport,
}

impl LoopbackPair {
    /// Creates a new loopback transport pair with bidirectional channels.
    pub fn new() -> Self {
        // Client-to-server channel
        let (c2s_tx, c2s_rx) = unbounded_channel();
        // Server-to-client channel
        let (s2c_tx, s2c_rx) = unbounded_channel();

        // Shared connection state
        let connected = Arc::new(AtomicBool::new(false));

        Self {
            client: LoopbackClientTransport {
                send: c2s_tx,
                recv: s2c_rx,
                connected: connected.clone(),
                event_tx: None,
            },
            server: LoopbackServerTransport {
                send: s2c_tx,
                recv: c2s_rx,
                connected,
                client_id: ClientId::default(),
                event_tx: None,
            },
        }
    }
}

impl Default for LoopbackPair {
    fn default() -> Self {
        Self::new()
    }
}

/// Client-side loopback transport.
///
/// Implements `ClientTransport` trait for in-memory communication with a server.
pub struct LoopbackClientTransport {
    send: UnboundedSender<OutgoingMessage>,
    recv: UnboundedReceiver<OutgoingMessage>,
    connected: Arc<AtomicBool>,
    event_tx: Option<UnboundedSender<ClientEvent>>,
}

impl LoopbackClientTransport {
    /// Connects the loopback transport and starts processing messages.
    pub fn connect(&mut self, events: UnboundedSender<ClientEvent>) -> Result<(), LoopbackError> {
        if self.connected.load(Ordering::SeqCst) {
            return Err(LoopbackError::AlreadyConnected);
        }

        self.event_tx = Some(events.clone());
        self.connected.store(true, Ordering::SeqCst);

        // Notify connection established
        let _ = events.send(ClientEvent::Connected { client_id: None });

        Ok(())
    }

    /// Disconnects from the loopback server.
    pub fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), LoopbackError> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(LoopbackError::NotConnected);
        }

        self.connected.store(false, Ordering::SeqCst);

        if let Some(tx) = &self.event_tx {
            let _ = tx.send(ClientEvent::Disconnected { reason });
        }

        Ok(())
    }

    /// Sends a message to the server.
    pub fn send(&self, message: OutgoingMessage) -> Result<(), LoopbackError> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(LoopbackError::NotConnected);
        }

        self.send
            .send(message)
            .map_err(|_| LoopbackError::ChannelClosed)
    }

    /// Sends a datagram (treated same as send for loopback).
    pub fn send_datagram(&self, payload: Bytes) -> Result<(), LoopbackError> {
        self.send(OutgoingMessage::new(0, payload))
    }

    /// Polls for incoming messages from the server.
    ///
    /// Should be called regularly to process received messages.
    pub fn poll_events(&mut self) -> Vec<ClientEvent> {
        let mut events = Vec::new();

        while let Ok(message) = self.recv.try_recv() {
            events.push(ClientEvent::Message {
                channel: message.channel,
                payload: message.payload,
            });
        }

        events
    }

    /// Returns the transport capabilities.
    pub fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            supports_reliable_streams: true,
            supports_unreliable_streams: false,
            supports_datagrams: true,
            max_channels: 255,
        }
    }
}

impl std::fmt::Debug for LoopbackClientTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopbackClientTransport")
            .field("connected", &self.connected.load(Ordering::SeqCst))
            .finish()
    }
}

/// Server-side loopback transport.
///
/// Implements `ServerTransport` trait for in-memory communication with a single client.
pub struct LoopbackServerTransport {
    send: UnboundedSender<OutgoingMessage>,
    recv: UnboundedReceiver<OutgoingMessage>,
    connected: Arc<AtomicBool>,
    client_id: ClientId,
    event_tx: Option<UnboundedSender<TransportEvent>>,
}

impl LoopbackServerTransport {
    /// Starts the loopback server transport.
    pub fn start(&mut self, events: UnboundedSender<TransportEvent>) -> Result<(), LoopbackError> {
        self.event_tx = Some(events.clone());
        self.connected.store(true, Ordering::SeqCst);

        // Notify that a client connected (the loopback client)
        let _ = events.send(TransportEvent::PeerConnected {
            client: self.client_id,
        });

        Ok(())
    }

    /// Stops the loopback server transport.
    pub fn stop(&mut self) {
        self.connected.store(false, Ordering::SeqCst);

        if let Some(tx) = &self.event_tx {
            let _ = tx.send(TransportEvent::PeerDisconnected {
                client: self.client_id,
                reason: DisconnectReason::Graceful,
            });
        }
    }

    /// Sends a message to the connected client.
    pub fn send(&self, client: ClientId, message: OutgoingMessage) -> Result<(), LoopbackError> {
        if client != self.client_id {
            return Err(LoopbackError::NotConnected);
        }

        if !self.connected.load(Ordering::SeqCst) {
            return Err(LoopbackError::NotConnected);
        }

        self.send
            .send(message)
            .map_err(|_| LoopbackError::ChannelClosed)
    }

    /// Sends a datagram to the connected client (treated same as send).
    pub fn send_datagram(&self, client: ClientId, payload: Bytes) -> Result<(), LoopbackError> {
        self.send(client, OutgoingMessage::new(0, payload))
    }

    /// Disconnects the specified client.
    pub fn disconnect(
        &self,
        client: ClientId,
        reason: DisconnectReason,
    ) -> Result<(), LoopbackError> {
        if client != self.client_id {
            return Err(LoopbackError::NotConnected);
        }

        self.connected.store(false, Ordering::SeqCst);

        if let Some(tx) = &self.event_tx {
            let _ = tx.send(TransportEvent::PeerDisconnected {
                client: self.client_id,
                reason,
            });
        }

        Ok(())
    }

    /// Polls for incoming messages from the client.
    ///
    /// Should be called regularly to process received messages.
    pub fn poll_events(&mut self) -> Vec<TransportEvent> {
        let mut events = Vec::new();

        while let Ok(message) = self.recv.try_recv() {
            events.push(TransportEvent::Message {
                client: self.client_id,
                channel: message.channel,
                payload: message.payload,
            });
        }

        events
    }

    /// Returns the transport capabilities.
    pub fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            supports_reliable_streams: true,
            supports_unreliable_streams: false,
            supports_datagrams: true,
            max_channels: 255,
        }
    }
}

impl std::fmt::Debug for LoopbackServerTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopbackServerTransport")
            .field("client_id", &self.client_id)
            .field("connected", &self.connected.load(Ordering::SeqCst))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_loopback_pair_creation() {
        let pair = LoopbackPair::new();
        assert!(!pair.client.connected.load(Ordering::SeqCst));
        assert!(!pair.server.connected.load(Ordering::SeqCst));
    }

    #[test]
    fn test_client_to_server_message() {
        let mut pair = LoopbackPair::new();

        // Setup event channels
        let (client_event_tx, mut _client_event_rx) = unbounded_channel();
        let (server_event_tx, mut _server_event_rx) = unbounded_channel();

        // Connect
        pair.client.connect(client_event_tx).unwrap();
        pair.server.start(server_event_tx).unwrap();

        // Send message from client to server
        let msg = OutgoingMessage::new(0, Bytes::from("Hello Server"));
        pair.client.send(msg.clone()).unwrap();

        // Server should receive the message
        let server_events = pair.server.poll_events();
        assert_eq!(server_events.len(), 1);

        if let TransportEvent::Message {
            client,
            channel,
            payload,
        } = &server_events[0]
        {
            assert_eq!(*client, 0);
            assert_eq!(*channel, 0);
            assert_eq!(payload.as_ref(), b"Hello Server");
        } else {
            panic!("Expected Message event");
        }
    }

    #[test]
    fn test_server_to_client_message() {
        let mut pair = LoopbackPair::new();

        let (client_event_tx, mut _client_event_rx) = unbounded_channel();
        let (server_event_tx, mut _server_event_rx) = unbounded_channel();

        pair.client.connect(client_event_tx).unwrap();
        pair.server.start(server_event_tx).unwrap();

        // Send message from server to client
        let msg = OutgoingMessage::new(0, Bytes::from("Hello Client"));
        pair.server.send(0, msg.clone()).unwrap();

        // Client should receive the message
        let client_events = pair.client.poll_events();
        assert_eq!(client_events.len(), 1);

        if let ClientEvent::Message { channel, payload } = &client_events[0] {
            assert_eq!(*channel, 0);
            assert_eq!(payload.as_ref(), b"Hello Client");
        } else {
            panic!("Expected Message event");
        }
    }

    #[test]
    fn test_bidirectional_communication() {
        let mut pair = LoopbackPair::new();

        let (client_event_tx, mut _client_event_rx) = unbounded_channel();
        let (server_event_tx, mut _server_event_rx) = unbounded_channel();

        pair.client.connect(client_event_tx).unwrap();
        pair.server.start(server_event_tx).unwrap();

        // Multiple messages in both directions
        for i in 0..5 {
            pair.client
                .send(OutgoingMessage::new(0, Bytes::from(format!("C2S {}", i))))
                .unwrap();

            pair.server
                .send(
                    0,
                    OutgoingMessage::new(0, Bytes::from(format!("S2C {}", i))),
                )
                .unwrap();
        }

        let server_events = pair.server.poll_events();
        assert_eq!(server_events.len(), 5);

        let client_events = pair.client.poll_events();
        assert_eq!(client_events.len(), 5);
    }

    #[test]
    fn test_disconnect() {
        let mut pair = LoopbackPair::new();

        let (client_event_tx, mut _client_event_rx) = unbounded_channel();
        let (server_event_tx, mut _server_event_rx) = unbounded_channel();

        pair.client.connect(client_event_tx).unwrap();
        pair.server.start(server_event_tx).unwrap();

        assert!(pair.client.connected.load(Ordering::SeqCst));

        // Disconnect client
        pair.client.disconnect(DisconnectReason::Graceful).unwrap();

        assert!(!pair.client.connected.load(Ordering::SeqCst));

        // Sending after disconnect should fail
        let result = pair
            .client
            .send(OutgoingMessage::new(0, Bytes::from("test")));
        assert!(result.is_err());
    }

    #[test]
    fn test_capabilities() {
        let pair = LoopbackPair::new();

        let client_caps = pair.client.capabilities();
        assert!(client_caps.supports_reliable_streams);
        assert!(client_caps.supports_datagrams);

        let server_caps = pair.server.capabilities();
        assert!(server_caps.supports_reliable_streams);
        assert!(server_caps.supports_datagrams);
    }
}
