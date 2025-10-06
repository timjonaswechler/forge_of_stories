use bevy_ecs::prelude::*;

use crate::{
    steam::{SteamAuthTicket, SteamDiscoveryEvent},
    ClientId,
};

/// Client-side Steam discovery updates surfaced to gameplay.
#[derive(Event, Debug, Clone)]
pub struct SteamClientDiscoveryEvent(pub SteamDiscoveryEvent);

/// Client-side authentication ticket lifecycle events.
#[derive(Event, Debug, Clone)]
pub enum SteamClientAuthEvent {
    TicketReady(SteamAuthTicket),
    TicketRejected(String),
    TicketValidated {
        client: Option<ClientId>,
        steam_id: u64,
        owner_steam_id: u64,
        result: Result<(), String>,
    },
}

/// Generic Steam client error surfaced from the transport.
#[derive(Event, Debug, Clone)]
pub struct SteamClientErrorEvent(pub String);

/// Server-side authentication results emitted after ticket validation.
#[derive(Event, Debug, Clone)]
pub struct SteamServerAuthEvent {
    pub client: Option<ClientId>,
    pub steam_id: u64,
    pub owner_steam_id: u64,
    pub result: Result<(), String>,
}

/// Server-side steam errors (e.g. malformed control packets, validation failures).
#[derive(Event, Debug, Clone)]
pub struct SteamServerErrorEvent {
    pub client: Option<ClientId>,
    pub steam_id: Option<u64>,
    pub error: String,
}
