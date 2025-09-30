//! Channel-basiertes Steam Discovery Backend.
//!
//! Dient als Brücke zwischen externen Steamworks/Aeronet-Integrationen und dem
//! generischen `SteamDiscoveryController`. Externe Systeme erhalten einen
//! `SteamBackendHandle`, über den sie `SteamServerEvent`s einspeisen können.

use std::sync::Arc;

use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use network_shared::discovery::SteamServerEvent;

use super::{SteamDiscoveryBackend, SteamDiscoveryError, SteamServerEventSender};

/// Backend, das Ereignisse über eine Tokio-MPSC-Queue entgegennimmt.
pub struct ChannelSteamDiscoveryBackend {
    rx: Arc<Mutex<mpsc::UnboundedReceiver<SteamServerEvent>>>,
    forwarder: Option<JoinHandle<()>>,
    event_sink: Option<SteamServerEventSender>,
}

impl ChannelSteamDiscoveryBackend {
    pub fn new() -> (Self, SteamBackendHandle) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                rx: Arc::new(Mutex::new(rx)),
                forwarder: None,
                event_sink: None,
            },
            SteamBackendHandle { sender: tx },
        )
    }
}

impl SteamDiscoveryBackend for ChannelSteamDiscoveryBackend {
    fn set_event_sink(&mut self, sender: SteamServerEventSender) {
        self.event_sink = Some(sender);
    }

    fn activate(&mut self) -> Result<(), SteamDiscoveryError> {
        if self.forwarder.is_some() {
            return Ok(());
        }
        let rx = self.rx.clone();
        let sink = self.event_sink.clone().ok_or_else(|| {
            SteamDiscoveryError::Backend("missing event sink for channel backend".into())
        })?;
        let handle = tokio::spawn(async move {
            loop {
                let event = {
                    let mut guard = rx.lock().await;
                    guard.recv().await
                };
                match event {
                    Some(event) => {
                        let _ = sink.send(event);
                    }
                    None => break,
                }
            }
        });
        self.forwarder = Some(handle);
        Ok(())
    }

    fn deactivate(&mut self) {
        if let Some(handle) = self.forwarder.take() {
            handle.abort();
        }
    }
}

/// Handle, das externen Code Zugriff auf die Event-Queue gibt.
#[derive(Clone)]
pub struct SteamBackendHandle {
    sender: mpsc::UnboundedSender<SteamServerEvent>,
}

impl SteamBackendHandle {
    pub fn send(&self, event: SteamServerEvent) {
        let _ = self.sender.send(event);
    }
}
