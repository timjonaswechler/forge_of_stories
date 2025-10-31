// scenes/in_game/cameras/mod.rs

mod first_person;
// mod pan_orbit;

use crate::GameState;
use crate::ui::cameras::cursor::CursorState;
use crate::utils::cleanup;
use bevy::prelude::*;

pub struct InGameCamerasPlugin;

/// Current in-game camera mode
#[derive(Resource, Default, PartialEq, Eq)]
pub enum InGameCameraMode {
    #[default]
    FirstPerson,
    // PanOrbit,
}

impl Plugin for InGameCamerasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InGameCameraMode>()
            .add_systems(OnEnter(GameState::InGame), spawn_camera)
            .add_systems(
                Update,
                (
                    first_person::handle_mouse_look,
                    first_person::follow_player,
                    first_person::apply_orientation,
                )
                    .chain()
                    .run_if(in_state(GameState::InGame))
                    .run_if(|mode: Res<InGameCameraMode>| *mode == InGameCameraMode::FirstPerson),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (cleanup::<InGameCamera>, release_cursor),
            );
    }
}

#[derive(Component)]
pub struct InGameCamera;

fn spawn_camera(mut commands: Commands, mut cursor: ResMut<CursorState>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 0.0).looking_at(Vec3::new(0.0, 1.7, -1.0), Vec3::Y),
        InGameCamera,
        first_person::FirstPersonView::default(),
        Name::new("InGame Camera"),
    ));

    // Lock cursor for FPS
    *cursor = CursorState::LOCKED;
}

fn release_cursor(mut cursor: ResMut<CursorState>) {
    *cursor = CursorState::FREE;
}
