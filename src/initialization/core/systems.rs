// src/initialization/core/systems.rs
use bevy::prelude::*;

// Startup-System zum Erstellen der UI-Kamera
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        // Diese Komponente ist wichtig, damit die UI weiß, welche Kamera sie verwenden soll!
        IsDefaultUiCamera,
    ));
    info!("Spawned default UI camera.");
}
