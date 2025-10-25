use super::defaults::CameraDefaults;
use crate::GameState;
use crate::client::LocalPlayer;
use crate::utils::cleanup;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin as POCP, TouchControls};
use game_server::components::Position;

/// Plugin for managing the pan-orbit camera in-game
/// This is the default camera for the InGame state
pub struct PanOrbitCameraPlugin;

impl Plugin for PanOrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(POCP)
            .add_systems(OnEnter(GameState::InGame), setup)
            .add_systems(
                Update,
                follow_local_player_focus.run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<PanOrbitCamera>);
    }
}

fn setup(mut commands: Commands, defaults: Res<CameraDefaults>) {
    commands.spawn((
        // WICHTIG: echte Kamera + Transform
        Camera3d::default(),
        Transform::default(),
        PanOrbitCamera {
            // Defaults: yaw=0 faces -Z, pitch>0 looks down (-Y)
            focus: Vec3::new(0.0, defaults.pan_orbit.focus_height, 0.0),
            yaw: Some(defaults.pan_orbit.yaw),
            pitch: Some(defaults.pan_orbit.pitch),
            radius: Some(defaults.pan_orbit.radius),

            // Limits
            pitch_upper_limit: Some(defaults.pan_orbit.pitch_max),
            pitch_lower_limit: Some(defaults.pan_orbit.pitch_min),
            zoom_upper_limit: Some(defaults.pan_orbit.zoom_max),
            zoom_lower_limit: defaults.pan_orbit.zoom_min,

            // Sensitivitäten
            orbit_sensitivity: defaults.pan_orbit.orbit_sensitivity,
            pan_sensitivity: defaults.pan_orbit.pan_sensitivity,
            zoom_sensitivity: defaults.pan_orbit.zoom_sensitivity,

            // Komfort
            allow_upside_down: true,
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Middle,
            modifier_pan: Some(KeyCode::ShiftLeft),
            reversed_zoom: true,
            touch_controls: TouchControls::TwoFingerOrbit,
            ..default()
        },
        Name::new("Pan-Orbit Camera"),
    ));
}

/// Fokus dynamisch an LocalPlayer ausrichten
fn follow_local_player_focus(
    local_player: Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<PanOrbitCamera>),
    >,
    mut cameras: Query<&mut PanOrbitCamera>,
    defaults: Res<CameraDefaults>,
) {
    let Ok((transform, position)) = local_player.single() else {
        return;
    };
    let base_translation = transform
        .map(|t| t.translation)
        .or_else(|| position.map(|p| p.translation))
        .unwrap_or(Vec3::ZERO);

    let focus = base_translation + Vec3::new(0.0, defaults.pan_orbit.focus_height, 0.0);

    for mut cam in &mut cameras {
        cam.focus = focus;
        cam.target_focus = focus;
    }
}
