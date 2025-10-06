use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use shared::{
    bevy::{SteamServerAuthEvent, SteamServerErrorEvent},
    QuinnetSyncUpdate, TransportEvent,
};

use crate::{update_sync_server, ConnectionEvent, ConnectionLostEvent, QuinnetServer};

#[derive(Resource)]
pub struct SteamServerEventChannel {
    sender: UnboundedSender<TransportEvent>,
    receiver: Arc<Mutex<UnboundedReceiver<TransportEvent>>>,
}

impl Default for SteamServerEventChannel {
    fn default() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

impl SteamServerEventChannel {
    pub fn sender(&self) -> UnboundedSender<TransportEvent> {
        self.sender.clone()
    }
}

pub struct QuinnetServerPlugin {
    pub initialize_later: bool,
}

impl Default for QuinnetServerPlugin {
    fn default() -> Self {
        Self {
            initialize_later: false,
        }
    }
}

impl Plugin for QuinnetServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectionEvent>()
            .add_event::<ConnectionLostEvent>();

        if !self.initialize_later {
            app.init_resource::<QuinnetServer>();
        }

        app.add_systems(
            PreUpdate,
            update_sync_server
                .in_set(QuinnetSyncUpdate)
                .run_if(resource_exists::<QuinnetServer>),
        );
    }
}

pub struct SteamworksServerPlugin {
    pub initialize_later: bool,
}

impl Default for SteamworksServerPlugin {
    fn default() -> Self {
        Self {
            initialize_later: false,
        }
    }
}

impl Plugin for SteamworksServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SteamServerAuthEvent>()
            .add_event::<SteamServerErrorEvent>();

        if !self.initialize_later && !app.world().contains_resource::<SteamServerEventChannel>() {
            app.insert_resource(SteamServerEventChannel::default());
        }

        app.add_systems(
            PreUpdate,
            pump_steam_server_events.run_if(resource_exists::<SteamServerEventChannel>),
        );
    }
}

fn pump_steam_server_events(
    channel: Res<SteamServerEventChannel>,
    mut auth_writer: EventWriter<SteamServerAuthEvent>,
    mut error_writer: EventWriter<SteamServerErrorEvent>,
) {
    let mut receiver = channel
        .receiver
        .lock()
        .expect("steam server event receiver poisoned");
    while let Ok(event) = receiver.try_recv() {
        match event {
            TransportEvent::AuthResult {
                client,
                steam_id,
                owner_steam_id,
                result,
            } => {
                auth_writer.write(SteamServerAuthEvent {
                    client,
                    steam_id,
                    owner_steam_id,
                    result,
                });
            }
            TransportEvent::Error { client, error } => {
                error_writer.write(SteamServerErrorEvent {
                    client,
                    steam_id: None,
                    error: error.to_string(),
                });
            }
            TransportEvent::PeerConnected { .. }
            | TransportEvent::PeerDisconnected { .. }
            | TransportEvent::Message { .. }
            | TransportEvent::Datagram { .. } => {
                // handled elsewhere in gameplay/transport layers
            }
        }
    }
}
