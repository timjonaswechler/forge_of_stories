use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use shared::{
    ClientEvent, QuinnetSyncUpdate,
    bevy::{SteamClientAuthEvent, SteamClientDiscoveryEvent, SteamClientErrorEvent},
    steam::SteamDiscoveryEvent,
};

use crate::{
    CertConnectionAbortEvent, CertInteractionEvent, CertTrustUpdateEvent, ConnectionEvent,
    ConnectionFailedEvent, ConnectionLostEvent, QuinnetClient, update_sync_client,
};

#[derive(Resource)]
pub struct SteamClientEventChannel {
    sender: UnboundedSender<ClientEvent>,
    receiver: Arc<Mutex<UnboundedReceiver<ClientEvent>>>,
}

impl Default for SteamClientEventChannel {
    fn default() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

impl SteamClientEventChannel {
    pub fn sender(&self) -> UnboundedSender<ClientEvent> {
        self.sender.clone()
    }
}

pub struct QuinnetClientPlugin {
    pub initialize_later: bool,
}

impl Default for QuinnetClientPlugin {
    fn default() -> Self {
        Self {
            initialize_later: false,
        }
    }
}

impl Plugin for QuinnetClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConnectionEvent>()
            .add_event::<ConnectionFailedEvent>()
            .add_event::<ConnectionLostEvent>()
            .add_event::<CertInteractionEvent>()
            .add_event::<CertTrustUpdateEvent>()
            .add_event::<CertConnectionAbortEvent>();

        if !self.initialize_later {
            app.init_resource::<QuinnetClient>();
        }

        app.add_systems(
            PreUpdate,
            update_sync_client
                .in_set(QuinnetSyncUpdate)
                .run_if(resource_exists::<QuinnetClient>),
        );
    }
}

pub struct SteamworksClientPlugin {
    pub initialize_later: bool,
}

impl Default for SteamworksClientPlugin {
    fn default() -> Self {
        Self {
            initialize_later: false,
        }
    }
}

impl Plugin for SteamworksClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SteamClientDiscoveryEvent>()
            .add_event::<SteamClientAuthEvent>()
            .add_event::<SteamClientErrorEvent>();

        if !self.initialize_later && !app.world().contains_resource::<SteamClientEventChannel>() {
            app.insert_resource(SteamClientEventChannel::default());
        }

        app.add_systems(
            PreUpdate,
            pump_steam_client_events.run_if(resource_exists::<SteamClientEventChannel>),
        );
    }
}

fn pump_steam_client_events(
    channel: Res<SteamClientEventChannel>,
    mut discovery_writer: EventWriter<SteamClientDiscoveryEvent>,
    mut auth_writer: EventWriter<SteamClientAuthEvent>,
    mut error_writer: EventWriter<SteamClientErrorEvent>,
) {
    let mut receiver = channel
        .receiver
        .lock()
        .expect("steam client event receiver poisoned");
    while let Ok(event) = receiver.try_recv() {
        match event {
            ClientEvent::Discovery(discovery) => match discovery {
                SteamDiscoveryEvent::AuthTicketReceived(ticket) => {
                    auth_writer.write(SteamClientAuthEvent::TicketReady(ticket));
                }
                SteamDiscoveryEvent::AuthTicketRejected(error) => {
                    auth_writer.write(SteamClientAuthEvent::TicketRejected(error));
                }
                other => {
                    discovery_writer.write(SteamClientDiscoveryEvent(other));
                }
            },
            ClientEvent::Error { error } => {
                error_writer.write(SteamClientErrorEvent(error.to_string()));
            }
            ClientEvent::Disconnected { reason } => {
                error_writer.write(SteamClientErrorEvent(format!(
                    "steam transport disconnected: {:?}",
                    reason
                )));
            }
            ClientEvent::AuthResult {
                client,
                steam_id,
                owner_steam_id,
                result,
            } => {
                auth_writer.write(SteamClientAuthEvent::TicketValidated {
                    client,
                    steam_id,
                    owner_steam_id,
                    result,
                });
            }
            ClientEvent::Connected { .. }
            | ClientEvent::Message { .. }
            | ClientEvent::Datagram { .. } => {
                // Handled by gameplay/transport layers elsewhere if needed.
            }
        }
    }
}
