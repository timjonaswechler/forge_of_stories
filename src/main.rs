use bevy::prelude::*;

mod character;
mod civilization;
mod narrative;
mod simulation;
mod utils;
mod world;

use simulation::time::TimeControlCommands;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Narrative World Simulator".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        // Add our game systems
        .add_plugins(world::WorldPlugin)
        .add_plugins(simulation::SimulationPlugin)
        .add_plugins(TimeControlCommands)
        .add_plugins(civilization::CivilizationPlugin)
        .add_plugins(character::CharacterPlugin)
        .add_plugins(narrative::NarrativePlugin)
        // Add startup systems
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Setup camera
    commands.spawn(Camera2d);

    // Add any initial entities or resources
    commands.insert_resource(simulation::SimulationState::default());
}
