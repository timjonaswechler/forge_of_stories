//! Integration tests for the loopback transport using the public trait API.

use bytes::Bytes;
use shared::transport::{LoopbackPair, TransportPayload};
use shared::{ClientEvent, DisconnectReason, TransportEvent};

const LOOPBACK_CLIENT: uuid::Uuid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");

fn connect_pair() -> LoopbackPair {
    let mut pair = LoopbackPair::new();
    pair.client.connect(()).expect("loopback connect");

    // Drain the initial connect events so each test starts with clear queues.
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
        [TransportEvent::PeerConnected { client }] if *client == LOOPBACK_CLIENT
    ));

    pair
}

#[test]
fn loopback_roundtrip() {
    let mut pair = connect_pair();

    pair.client
        .send(TransportPayload::message(0, Bytes::from_static(b"hello")))
        .unwrap();

    let mut server_events = Vec::new();
    pair.server.poll_events(&mut server_events);
    assert!(matches!(
        server_events.as_slice(),
        [TransportEvent::Message { client, channel, payload }]
        if *client == LOOPBACK_CLIENT && *channel == 0 && payload.as_ref() == b"hello"
    ));

    pair.server
        .send(
            LOOPBACK_CLIENT,
            TransportPayload::message(1, Bytes::from_static(b"reply")),
        )
        .unwrap();

    let mut client_events = Vec::new();
    pair.client.poll_events(&mut client_events);
    assert!(matches!(
        client_events.as_slice(),
        [ClientEvent::Message { channel, payload }]
        if *channel == 1 && payload.as_ref() == b"reply"
    ));
}

#[test]
fn loopback_message_order_is_preserved() {
    let mut pair = connect_pair();

    for i in 0..50 {
        pair.client
            .send(TransportPayload::message(
                0,
                Bytes::from(format!("C2S {i}")),
            ))
            .unwrap();
        pair.server
            .send(
                LOOPBACK_CLIENT,
                TransportPayload::message(0, Bytes::from(format!("S2C {i}"))),
            )
            .unwrap();
    }

    let mut server_events = Vec::new();
    pair.server.poll_events(&mut server_events);
    assert_eq!(server_events.len(), 50);
    for (idx, event) in server_events.into_iter().enumerate() {
        match event {
            TransportEvent::Message { payload, .. } => {
                assert_eq!(payload.as_ref(), format!("C2S {idx}").as_bytes());
            }
            other => panic!("unexpected server event: {other:?}"),
        }
    }

    let mut client_events = Vec::new();
    pair.client.poll_events(&mut client_events);
    assert_eq!(client_events.len(), 50);
    for (idx, event) in client_events.into_iter().enumerate() {
        match event {
            ClientEvent::Message { payload, .. } => {
                assert_eq!(payload.as_ref(), format!("S2C {idx}").as_bytes());
            }
            other => panic!("unexpected client event: {other:?}"),
        }
    }
}

#[test]
fn loopback_disconnect_propagates() {
    let mut pair = connect_pair();

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
        if *client == LOOPBACK_CLIENT
    ));
}
