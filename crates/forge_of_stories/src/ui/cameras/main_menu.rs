// DEPRECATED: MainMenu nutzt jetzt die zentrale SceneCamera über CameraModeChangeEvent
// Dieses Modul kann später gelöscht werden, wenn CameraPanEvent-Logic woanders implementiert ist.

use bevy::prelude::*;

/// Event to trigger camera pan animation in main menu
/// TODO: In Zukunft über CameraModeChangeEvent mit Animation-Flag
#[derive(Event)]
pub struct CameraPanEvent {
    pub target_position: Vec3,
    pub target_look_at: Vec3,
}
