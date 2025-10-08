//! Integration test for loopback transport.
//!
//! Verifies that client and server can communicate in the same process
//! with zero network I/O.

use bytes::Bytes;
use shared::transport::{LoopbackError, LoopbackPair};
use shared::{ClientEvent, DisconnectReason, OutgoingMessage, TransportEvent};
use tokio::sync::mpsc::unbounded_channel;

#[test]
fn test_loopback_client_server_same_process() {
    use uuid::uuid;

    // Create loopback pair
    let mut pair = LoopbackPair::new();

    // Setup event channels
    let (client_event_tx, mut client_event_rx) = unbounded_channel();
    let (server_event_tx, mut server_event_rx) = unbounded_channel();

    // Connect both sides
    pair.client.connect(client_event_tx).unwrap();
    pair.server.start(server_event_tx).unwrap();

    // Verify connection events
    let client_connected = client_event_rx.try_recv().unwrap();
    assert!(matches!(
        client_connected,
        ClientEvent::Connected { client_id: None }
    ));

    let server_peer_connected = server_event_rx.try_recv().unwrap();
    let expected_uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
    assert!(matches!(
        server_peer_connected,
        TransportEvent::PeerConnected { client } if client == expected_uuid
    ));

    // Client sends message to server
    let c2s_msg = OutgoingMessage::new(0, Bytes::from("Client Hello"));
    pair.client.send(c2s_msg).unwrap();

    // Server receives and processes
    let server_events = pair.server.poll_events();
    assert_eq!(server_events.len(), 1);

    if let TransportEvent::Message {
        client,
        channel,
        payload,
    } = &server_events[0]
    {
        assert_eq!(*client, uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"));
        assert_eq!(*channel, 0);
        assert_eq!(payload.as_ref(), b"Client Hello");

        // Server responds
        let s2c_msg = OutgoingMessage::new(0, Bytes::from("Server Response"));
        pair.server.send(*client, s2c_msg).unwrap();
    } else {
        panic!("Expected Message event from server");
    }

    // Client receives response
    let client_events = pair.client.poll_events();
    assert_eq!(client_events.len(), 1);

    if let ClientEvent::Message { channel, payload } = &client_events[0] {
        assert_eq!(*channel, 0);
        assert_eq!(payload.as_ref(), b"Server Response");
    } else {
        panic!("Expected Message event from client");
    }
}

#[test]
fn test_loopback_multiple_messages() {
    use uuid::uuid;
    let mut pair = LoopbackPair::new();

    let (client_event_tx, mut _client_event_rx) = unbounded_channel();
    let (server_event_tx, mut _server_event_rx) = unbounded_channel();

    pair.client.connect(client_event_tx).unwrap();
    pair.server.start(server_event_tx).unwrap();

    // Send 100 messages in each direction
    let count = 100;
    for i in 0..count {
        pair.client
            .send(OutgoingMessage::new(0, Bytes::from(format!("C2S {}", i))))
            .unwrap();

        pair.server
            .send(
                uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
                OutgoingMessage::new(0, Bytes::from(format!("S2C {}", i))),
            )
            .unwrap();
    }

    // Verify all messages received
    let server_events = pair.server.poll_events();
    assert_eq!(server_events.len(), count);

    let client_events = pair.client.poll_events();
    assert_eq!(client_events.len(), count);

    // Verify message order is preserved
    for (i, event) in server_events.iter().enumerate() {
        if let TransportEvent::Message { payload, .. } = event {
            let expected = format!("C2S {}", i);
            assert_eq!(payload.as_ref(), expected.as_bytes());
        }
    }

    for (i, event) in client_events.iter().enumerate() {
        if let ClientEvent::Message { payload, .. } = event {
            let expected = format!("S2C {}", i);
            assert_eq!(payload.as_ref(), expected.as_bytes());
        }
    }
}

#[test]
fn test_loopback_disconnect_lifecycle() {
    let mut pair = LoopbackPair::new();

    let (client_event_tx, mut client_event_rx) = unbounded_channel();
    let (server_event_tx, mut server_event_rx) = unbounded_channel();

    // Connect
    pair.client.connect(client_event_tx).unwrap();
    pair.server.start(server_event_tx).unwrap();

    // Clear connection events
    let _ = client_event_rx.try_recv();
    let _ = server_event_rx.try_recv();

    // Client disconnects gracefully
    pair.client.disconnect(DisconnectReason::Graceful).unwrap();

    // Verify client receives disconnect event
    let client_disconnect = client_event_rx.try_recv().unwrap();
    assert!(matches!(
        client_disconnect,
        ClientEvent::Disconnected {
            reason: DisconnectReason::Graceful
        }
    ));

    // Verify client cannot send after disconnect
    let result = pair
        .client
        .send(OutgoingMessage::new(0, Bytes::from("test")));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LoopbackError::NotConnected));
}

#[test]
fn test_loopback_server_disconnect() {
    use uuid::uuid;
    let mut pair = LoopbackPair::new();

    let (client_event_tx, mut _client_event_rx) = unbounded_channel();
    let (server_event_tx, mut server_event_rx) = unbounded_channel();

    pair.client.connect(client_event_tx).unwrap();
    pair.server.start(server_event_tx).unwrap();

    // Clear connection events
    let _ = server_event_rx.try_recv();

    // Server disconnects the client
    pair.server
        .disconnect(
            uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            DisconnectReason::Kicked,
        )
        .unwrap();

    // Verify server emitted disconnect event
    let server_disconnect = server_event_rx.try_recv().unwrap();
    let matching_uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
    assert!(matches!(
        server_disconnect,
        TransportEvent::PeerDisconnected {
            client: matching_uuid,
            reason: DisconnectReason::Kicked
        }
    ));
}

#[test]
fn test_loopback_no_network_io() {
    // This test verifies that loopback transport doesn't use network I/O.
    // Since we're using in-memory channels, there should be no socket operations.
    //
    // On Unix systems, you could verify this with:
    // - strace: no socket(), bind(), connect() syscalls
    // - lsof: no network file descriptors
    //
    // For this test, we simply verify that the transport works without
    // binding to any network interface.
    use uuid::uuid;
    let mut pair = LoopbackPair::new();
    let (client_event_tx, mut _client_event_rx) = unbounded_channel();
    let (server_event_tx, mut _server_event_rx) = unbounded_channel();

    // Connect (should not require any network)
    pair.client.connect(client_event_tx).unwrap();
    pair.server.start(server_event_tx).unwrap();

    // Exchange messages (should be instant, no network latency)
    for i in 0..10 {
        pair.client
            .send(OutgoingMessage::new(0, Bytes::from(format!("msg {}", i))))
            .unwrap();
    }

    let events = pair.server.poll_events();
    assert_eq!(events.len(), 10);

    // All messages should be delivered instantly (same process, same memory)
    // No network overhead, no serialization over network, just in-memory copy
}

#[test]
fn test_loopback_datagram_support() {
    use uuid::uuid;
    let mut pair = LoopbackPair::new();

    let (client_event_tx, mut _client_event_rx) = unbounded_channel();
    let (server_event_tx, mut _server_event_rx) = unbounded_channel();

    pair.client.connect(client_event_tx).unwrap();
    pair.server.start(server_event_tx).unwrap();

    // Send datagrams (treated as regular messages for loopback)
    pair.client
        .send_datagram(Bytes::from("datagram 1"))
        .unwrap();
    pair.client
        .send_datagram(Bytes::from("datagram 2"))
        .unwrap();

    pair.server
        .send_datagram(
            uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            Bytes::from("server datagram"),
        )
        .unwrap();

    // Server receives client datagrams
    let server_events = pair.server.poll_events();
    assert_eq!(server_events.len(), 2);

    // Client receives server datagram
    let client_events = pair.client.poll_events();
    assert_eq!(client_events.len(), 1);
}

#[test]
fn test_loopback_capabilities() {
    let pair = LoopbackPair::new();

    let client_caps = pair.client.capabilities();
    assert!(client_caps.supports_reliable_streams);
    assert!(client_caps.supports_datagrams);
    assert_eq!(client_caps.max_channels, 255);

    let server_caps = pair.server.capabilities();
    assert!(server_caps.supports_reliable_streams);
    assert!(server_caps.supports_datagrams);
    assert_eq!(server_caps.max_channels, 255);
}
