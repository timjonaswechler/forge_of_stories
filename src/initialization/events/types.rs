// src/initialization/events/types.rs
use bevy::prelude::*;

// Events für Zustandsänderungen in der Anwendung
#[derive(Event)]
pub struct AssetsLoadedEvent;

#[derive(Event)]
pub struct AppStartupCompletedEvent;
