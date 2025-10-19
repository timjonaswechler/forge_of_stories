use crate::GameState;
use crate::ui::normal_vector::NormalArrowVisual;
use bevy::{
    color::palettes::basic::{BLUE, GREEN, RED},
    prelude::*,
};
use game_server::{Player, PlayerInput, Position};
use networking::prelude::ClientState;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum NetSets {
    Receive,
    Reconcile,
    InputHandling,
    Prediction,
    Update,
    Transmit,
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                NetSets::Receive,
                NetSets::Reconcile,
                NetSets::InputHandling,
                NetSets::Prediction,
                NetSets::Update,
                NetSets::Transmit,
            )
                .chain()
                // Hier fügen Sie die `run_if` Bedingung für alle Sets hinzu
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(ClientState::Connected)),
        )
        // Input Handling Set
        .add_systems(
            Update,
            send_player_input
                .in_set(NetSets::InputHandling)
                .run_if(crate::ui::menu_closed),
        )
        // Prediction Set
        // .add_systems(Update, predict_player_position.in_set(NetSets::Prediction))
        // Update Set
        .add_systems(Update, update_player_positions.in_set(NetSets::Update))
        // Transmit Set
        // .add_systems(Update, transmit_player_position.in_set(NetSets::Transmit));
        // cleanup
        .add_systems(
            Update,
            crate::utils::cleanup::<DebugArrows>.run_if(in_state(GameState::InGame)),
        );
    }
}

/// System that sends player input to the server using bevy_replicon's Message system.
///
/// This system runs every frame when in-game and the menu is closed.
pub fn send_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    input_writer: Option<MessageWriter<PlayerInput>>,
) {
    // Return early if the message writer is not yet initialized
    let Some(mut input_writer) = input_writer else {
        return;
    };

    // Calculate movement direction from WASD/Arrow keys
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Normalize diagonal movement
    if direction.length() > 0.01 {
        direction = direction.normalize();
    }

    // Send input to server
    input_writer.write(PlayerInput {
        direction,
        jump: keyboard.pressed(KeyCode::Space),
    });
}

#[derive(Component)]
pub struct DebugArrows;

/// System that updates player mesh positions from replicated Position components.
///
/// This runs every frame to smoothly update visual positions.
pub fn update_player_positions(
    mut commands: Commands,
    mut players: Query<(&Position, &mut Transform), With<Player>>,
) {
    for (position, mut transform) in &mut players {
        transform.translation = position.translation;
        let head_origin = position.translation + Vec3::new(0.0, 1.0, 0.0);
        commands.spawn((
            NormalArrowVisual {
                origin: head_origin,
                direction: Vec3::Y, // Normale ist der normalisierte Vektor vom Kugelzentrum zum Punkt
                length: 0.5,
                color: RED,
            },
            crate::InGame,
            DebugArrows,
        ));

        // Pfeil am Körper (X-Achse)
        let body_origin = position.translation + Vec3::new(0.5, 0.0, 0.0);
        commands.spawn((
            NormalArrowVisual {
                origin: body_origin,
                direction: Vec3::X,
                length: 0.5,
                color: GREEN,
            },
            crate::InGame,
            DebugArrows,
        ));

        // Pfeil am Äquator (Z-Achse)
        let hip_origin = position.translation + Vec3::new(0.0, 0.0, 0.5);
        commands.spawn((
            NormalArrowVisual {
                origin: hip_origin,
                direction: Vec3::Z,
                length: 0.5,
                color: BLUE,
            },
            crate::InGame,
            DebugArrows,
        ));
    }
}
