use super::*;
use crate::client::LocalPlayer;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};
use bevy_tweening::{AnimCompletedEvent, Tween, TweenAnim, lens::Lens};
use game_server::components::Position;
use std::time::Duration;

// Imports aus first_person Modul
use super::first_person::{FirstPersonView, FollowLocalPlayer};

/// Component marking a camera that is currently transitioning between modes
#[derive(Component)]
pub struct CameraTransitioning {
    pub source_mode: CameraMode,
    pub target_mode: CameraMode,
}

const TRANSITION_DURATION: f32 = 0.6;

pub fn handle_camera_mode_changes(
    mut commands: Commands,
    mut events: MessageReader<CameraModeChangeEvent>,
    mut current_mode: ResMut<CameraMode>,
    mut transition_state: ResMut<CameraTransitionState>,
    mut camera_query: Query<(Entity, &mut Transform), With<SceneCamera>>,
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
    defaults: Res<CameraDefaults>,
    local_player_query: Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    for event in events.read() {
        let Ok((camera_entity, mut camera_transform)) = camera_query.single_mut() else {
            continue;
        };

        if event.animate {
            // Mit Transition: Nur Tween starten, Components NICHT wechseln
            // PanOrbitCamera deaktivieren wenn vorhanden (verhindert dass sie während Transition Transform manipuliert)
            if let Ok(mut pan_orbit) = pan_orbit_query.get_mut(camera_entity) {
                pan_orbit.enabled = false;
            }

            // Cursor-State sofort anpassen
            let grab = matches!(
                event.new_mode,
                CameraMode::InGame(InGameCameraMode::FirstPerson)
            );
            set_cursor_state(grab, &mut window_query);

            start_camera_transition(
                &mut commands,
                camera_entity,
                &camera_transform,
                &event.new_mode,
                &current_mode.clone(), // OLD mode mitgeben
                &defaults,
                &local_player_query,
                &mut transition_state,
            );
            // Mode und Components werden erst nach Transition-Completion geändert
        } else {
            // Instant: Sofort alles anwenden
            remove_mode_components(&mut commands, camera_entity, &current_mode);
            add_mode_components(
                &mut commands,
                camera_entity,
                &event.new_mode,
                &defaults,
                &local_player_query,
            );
            *camera_transform =
                calculate_transform_for_mode(&event.new_mode, &defaults, &local_player_query);
            *current_mode = event.new_mode.clone();
        }
    }
}

fn remove_mode_components(commands: &mut Commands, entity: Entity, mode: &CameraMode) {
    match mode {
        CameraMode::Splashscreen | CameraMode::MainMenu => {
            // Keine zusätzlichen Components
        }
        CameraMode::InGame(InGameCameraMode::FirstPerson) => {
            commands
                .entity(entity)
                .remove::<FirstPersonView>()
                .remove::<FollowLocalPlayer>();
        }
        CameraMode::InGame(InGameCameraMode::PanOrbit) => {
            commands.entity(entity).remove::<PanOrbitCamera>();
        }
    }
}

fn add_mode_components(
    commands: &mut Commands,
    entity: Entity,
    mode: &CameraMode,
    defaults: &CameraDefaults,
    local_player_query: &Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
) {
    match mode {
        CameraMode::Splashscreen | CameraMode::MainMenu => {
            // Keine zusätzlichen Components
        }
        CameraMode::InGame(InGameCameraMode::FirstPerson) => {
            commands.entity(entity).insert((
                FirstPersonView {
                    sensitivity: defaults.first_person.mouse_sensitivity,
                    ..Default::default()
                },
                FollowLocalPlayer {
                    offset: Vec3::new(0.0, defaults.first_person.height_offset, 0.0),
                },
            ));
        }
        CameraMode::InGame(InGameCameraMode::PanOrbit) => {
            // Berechne korrekten Focus basierend auf Player-Position
            let player_pos = get_player_translation(local_player_query);
            let focus = player_pos + Vec3::new(0.0, defaults.pan_orbit.focus_height, 0.0);

            commands.entity(entity).insert(PanOrbitCamera {
                focus,
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

                // Komfort-Settings (wie in pan_orbit.rs)
                allow_upside_down: true,
                button_orbit: MouseButton::Middle,
                button_pan: MouseButton::Middle,
                modifier_pan: Some(KeyCode::ShiftLeft),
                reversed_zoom: true,
                touch_controls: pan_orbit::TouchControls::TwoFingerOrbit,

                ..default()
            });
        }
    }
}

fn calculate_transform_for_mode(
    mode: &CameraMode,
    defaults: &CameraDefaults,
    local_player_query: &Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
) -> Transform {
    match mode {
        CameraMode::Splashscreen => Transform::from_translation(defaults.splashscreen.position)
            .looking_at(defaults.splashscreen.look_at, Vec3::Y),
        CameraMode::MainMenu => Transform::from_translation(defaults.main_menu.position)
            .looking_at(defaults.main_menu.look_at, Vec3::Y),
        CameraMode::InGame(InGameCameraMode::FirstPerson) => {
            // Player-Position holen + Offset
            let player_pos = get_player_translation(local_player_query);
            Transform::from_translation(
                player_pos + Vec3::new(0.0, defaults.first_person.height_offset, 0.0),
            )
        }
        CameraMode::InGame(InGameCameraMode::PanOrbit) => {
            // Berechne Transform basierend auf PanOrbit defaults
            let player_pos = get_player_translation(local_player_query);
            let focus = player_pos + Vec3::new(0.0, defaults.pan_orbit.focus_height, 0.0);
            transform_from_orbit(
                focus,
                defaults.pan_orbit.yaw,
                defaults.pan_orbit.pitch,
                defaults.pan_orbit.radius,
            )
        }
    }
}

fn get_player_translation(
    local_player_query: &Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
) -> Vec3 {
    let Ok((transform, position)) = local_player_query.single() else {
        return Vec3::ZERO;
    };

    transform
        .map(|t| t.translation)
        .or_else(|| position.map(|p| p.translation))
        .unwrap_or(Vec3::ZERO)
}

/// Berechnet die Transform für eine PanOrbit-Kamera basierend auf Orbit-Parametern
fn transform_from_orbit(focus: Vec3, yaw: f32, pitch: f32, radius: f32) -> Transform {
    // Sphärische Koordinaten → Kartesische Koordinaten
    let h = radius * pitch.cos();
    let x = h * yaw.sin();
    let z = h * yaw.cos();
    let y = radius * pitch.sin();

    Transform::from_translation(focus + Vec3::new(x, y, z)).looking_at(focus, Vec3::Y)
}

// Custom Lens für komplette Transform-Tweening
#[derive(Clone)]
struct TransformTweenLens {
    start: Transform,
    end: Transform,
}

impl Lens<Transform> for TransformTweenLens {
    fn lerp(&mut self, mut target: Mut<Transform>, ratio: f32) {
        target.translation = self.start.translation.lerp(self.end.translation, ratio);
        target.rotation = self.start.rotation.slerp(self.end.rotation, ratio);
        target.scale = self.start.scale.lerp(self.end.scale, ratio);
    }
}

fn start_camera_transition(
    commands: &mut Commands,
    camera_entity: Entity,
    current_transform: &Transform,
    target_mode: &CameraMode,
    source_mode: &CameraMode,
    defaults: &CameraDefaults,
    local_player_query: &Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
    transition_state: &mut ResMut<CameraTransitionState>,
) {
    let target_transform = calculate_transform_for_mode(target_mode, defaults, local_player_query);

    // Transform Tween
    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs_f32(TRANSITION_DURATION),
        TransformTweenLens {
            start: *current_transform,
            end: target_transform,
        },
    );

    commands.entity(camera_entity).insert((
        CameraTransitioning {
            source_mode: source_mode.clone(),
            target_mode: target_mode.clone(),
        },
        TweenAnim::new(tween),
    ));

    transition_state.active = true;
}

/// System that handles transition completion
pub fn handle_transition_completion(
    mut commands: Commands,
    mut completed_events: MessageReader<AnimCompletedEvent>,
    mut camera_mode: ResMut<CameraMode>,
    mut transition_state: ResMut<CameraTransitionState>,
    transitioning: Query<(Entity, &CameraTransitioning), With<SceneCamera>>,
    defaults: Res<CameraDefaults>,
    local_player_query: Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
) {
    for event in completed_events.read() {
        // Prüfe ob die Entity eine transitioning camera ist
        let Ok((camera_entity, transitioning)) = transitioning.get(event.anim_entity) else {
            continue;
        };

        // Alte Components vom SOURCE mode entfernen (nicht vom aktuellen camera_mode!)
        remove_mode_components(&mut commands, camera_entity, &transitioning.source_mode);

        // Neue Components für TARGET mode hinzufügen
        add_mode_components(
            &mut commands,
            camera_entity,
            &transitioning.target_mode,
            &defaults,
            &local_player_query,
        );

        // Mode updaten
        *camera_mode = transitioning.target_mode.clone();

        // Cleanup
        commands
            .entity(camera_entity)
            .remove::<CameraTransitioning>();
        transition_state.active = false;
    }
}
