// DEPRECATED: Dieses ganze Modul ist veraltet und muss neu geschrieben werden
// für das Single-Camera-System mit CameraMode

use super::{
    CameraDefaults, CameraMode, CameraModeChangeEvent, CameraTransitionState, InGameCameraMode,
    SceneCamera,
    first_person::{
        FirstPersonView, FollowLocalPlayer, first_person_transform_from_view,
        set_view_from_rotation,
    },
    pan_orbit::PanOrbitCamera,
    set_cursor_state,
};
use crate::{GameState, client::LocalPlayer};
use bevy::{
    math::curve::EaseFunction,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
use bevy_tweening::{AnimCompletedEvent, Tween, TweenAnim, TweeningPlugin, lens::Lens};
use game_server::components::Position;
use std::time::Duration;

const TRANSITION_DURATION: f32 = 0.6;

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

#[allow(dead_code)]
#[derive(Component, Clone)]
struct CameraTransition {
    to: CameraMode,
    additional: TransitionAdditional,
}

#[allow(dead_code)]
#[derive(Clone)]
enum TransitionAdditional {
    ToPanOrbit {
        pan_orbit_entity: Entity,
        previous_enabled: bool,
    },
    ToFirstPerson {
        first_person_entity: Entity,
        previous_enabled: bool,
    },
}

// TODO: Dieses System muss komplett neu geschrieben werden für Single-Camera
#[allow(dead_code)]
fn start_transitions(
    mut commands: Commands,
    mut toggle_events: MessageReader<CameraModeChangeEvent>,
    mut transition_state: ResMut<CameraTransitionState>,
    camera_mode: Res<CameraMode>,
    defaults: Res<CameraDefaults>,
    mut camera_queries: ParamSet<(
        Query<
            (
                Entity,
                &mut Transform,
                &FollowLocalPlayer,
                &mut FirstPersonView,
            ),
            With<SceneCamera>,
        >,
        Query<(Entity, &mut Transform, &mut PanOrbitCamera), With<PanOrbitCamera>>,
        Query<(Option<&Transform>, Option<&Position>), (With<LocalPlayer>, Without<SceneCamera>)>,
    )>,
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    if transition_state.active {
        return;
    }

    let mut triggered = false;
    for _ in toggle_events.read() {
        triggered = true;
    }
    if !triggered {
        return;
    }

    let player_translation = {
        let query = camera_queries.p2();
        local_player_translation(&query).unwrap_or(Vec3::ZERO)
    };

    match &*camera_mode {
        CameraMode::InGame(InGameCameraMode::FirstPerson) => {
            // Hole aktuelle FP-Transform
            let (fp_entity, fp_transform) = {
                let mut query = camera_queries.p0();
                let Ok((entity, transform, _follow, _view)) = query.single_mut() else {
                    return;
                };
                info!("Start FP -> PO: fp_start.pos={:?}", transform.translation);
                (entity, transform.clone())
            };

            // PanOrbit auf Defaults über Spieler setzen
            let (po_entity, po_target, previous_enabled) = {
                let mut query = camera_queries.p1();
                let Ok((entity, mut transform, mut pan_orbit)) = query.single_mut() else {
                    return;
                };

                let (target, focus) = pan_orbit_target_from_defaults(player_translation, &defaults);
                let previous_enabled = pan_orbit.enabled;
                pan_orbit.enabled = false;

                apply_pan_orbit_defaults(&mut pan_orbit, focus, &defaults);

                *transform = target.clone();
                info!(
                    "Start FP -> PO: po_target.pos={:?}, focus={:?}",
                    target.translation, focus
                );

                (entity, target, previous_enabled)
            };

            transition_state.active = true;
            set_cursor_state(false, &mut window_query);

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs_f32(TRANSITION_DURATION),
                TransformTweenLens {
                    start: fp_transform,
                    end: po_target,
                },
            );

            commands.entity(fp_entity).insert((
                CameraTransition {
                    to: CameraMode::InGame(InGameCameraMode::PanOrbit),
                    additional: TransitionAdditional::ToPanOrbit {
                        pan_orbit_entity: po_entity,
                        previous_enabled,
                    },
                },
                TweenAnim::new(tween),
            ));
        }
        CameraMode::InGame(InGameCameraMode::PanOrbit) => {
            // Hole aktuelle PanOrbit-Transform
            let (po_entity, po_transform, previous_enabled) = {
                let mut query = camera_queries.p1();
                let Ok((entity, transform, mut pan_orbit)) = query.single_mut() else {
                    return;
                };
                let current_transform = transform.clone();
                info!(
                    "Start PO -> FP: po_start.pos={:?}",
                    current_transform.translation
                );
                let previous_enabled = pan_orbit.enabled;
                pan_orbit.enabled = false;
                (entity, current_transform, previous_enabled)
            };

            // First-Person Defaults über Spieler setzen
            let (fp_entity, fp_target) = {
                let mut query = camera_queries.p0();
                let Ok((entity, mut transform, follow, mut view)) = query.single_mut() else {
                    return;
                };

                // Default-Rotation: geradeaus (Yaw=0, Pitch=0)
                set_view_from_rotation(&mut view, Quat::IDENTITY);

                let target = first_person_transform_from_view(player_translation, follow, &view);
                info!("Start PO -> FP: fp_target.pos={:?}", target.translation);

                *transform = target.clone();

                (entity, target)
            };

            transition_state.active = true;
            set_cursor_state(true, &mut window_query);

            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs_f32(TRANSITION_DURATION),
                TransformTweenLens {
                    start: po_transform,
                    end: fp_target,
                },
            );

            commands.entity(po_entity).insert((
                CameraTransition {
                    to: CameraMode::InGame(InGameCameraMode::FirstPerson),
                    additional: TransitionAdditional::ToFirstPerson {
                        first_person_entity: fp_entity,
                        previous_enabled,
                    },
                },
                TweenAnim::new(tween),
            ));
        }
        // Transitions sind nur für InGame-Modi relevant
        CameraMode::Splashscreen | CameraMode::MainMenu => {
            // Keine Transitions
        }
    }
}

#[allow(dead_code)]
fn finish_transitions(
    mut commands: Commands,
    mut completed: MessageReader<AnimCompletedEvent>,
    mut camera_mode: ResMut<CameraMode>,
    mut transition_state: ResMut<CameraTransitionState>,
    transitions: Query<&CameraTransition>,
    mut camera_query: Query<&mut Camera>,
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut po_cam_and_tf: Query<(&mut PanOrbitCamera, &mut Transform), With<PanOrbitCamera>>,
) {
    for event in completed.read() {
        let Ok(transition) = transitions.get(event.anim_entity) else {
            continue;
        };
        let transition = transition.clone();

        match transition.additional.clone() {
            TransitionAdditional::ToPanOrbit {
                pan_orbit_entity,
                previous_enabled,
            } => {
                // Vor Aktivierung: Transform aus Zielwerten rekonstruieren und setzen
                if let Ok((mut pan_orbit, mut tf)) = po_cam_and_tf.get_mut(pan_orbit_entity) {
                    let forced = compute_pan_orbit_transform_from(&pan_orbit);
                    info!(
                        "Camera switch FP -> PanOrbit: applying final transform: pos={:?}",
                        forced.translation
                    );
                    *tf = forced;
                    // Urspruenglichen Enabled-Status wiederherstellen
                    pan_orbit.enabled = previous_enabled;
                }

                if let Ok(mut cam) = camera_query.get_mut(event.anim_entity) {
                    cam.is_active = false;
                }
                if let Ok(mut cam) = camera_query.get_mut(pan_orbit_entity) {
                    cam.is_active = true;
                }
            }
            TransitionAdditional::ToFirstPerson {
                first_person_entity,
                previous_enabled,
            } => {
                if let Ok(mut cam) = camera_query.get_mut(event.anim_entity) {
                    cam.is_active = false;
                }
                if let Ok(mut cam) = camera_query.get_mut(first_person_entity) {
                    cam.is_active = true;
                }
                // Hier ist event.anim_entity die PanOrbit-Kamera (sie wurde getweened)
                if let Ok((mut pan_orbit, _tf)) = po_cam_and_tf.get_mut(event.anim_entity) {
                    pan_orbit.enabled = previous_enabled;
                }
            }
        }

        let grab = matches!(
            transition.to,
            CameraMode::InGame(InGameCameraMode::FirstPerson)
        );
        *camera_mode = transition.to;
        set_cursor_state(grab, &mut window_query);

        commands.entity(event.anim_entity).remove::<TweenAnim>();
        commands
            .entity(event.anim_entity)
            .remove::<CameraTransition>();
        transition_state.active = false;
    }
}

fn local_player_translation(
    query: &Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
) -> Option<Vec3> {
    let (transform, position) = query.single().ok()?;
    transform
        .map(|t| t.translation)
        .or_else(|| position.map(|p| p.translation))
}

fn pan_orbit_target_from_defaults(
    player_translation: Vec3,
    defaults: &CameraDefaults,
) -> (Transform, Vec3) {
    let focus = player_translation + Vec3::Y * defaults.pan_orbit.focus_height;
    let yaw = defaults.pan_orbit.yaw;
    let pitch = defaults.pan_orbit.pitch;
    let radius = defaults.pan_orbit.radius;

    // Debug: erwartete Default-Werte
    info!(
        "Compute PanOrbit target (defaults): player={:?}, focus={:?}, yaw={:.3}, pitch={:.3}, radius={:.3}",
        player_translation, focus, yaw, pitch, radius
    );

    let t = transform_from_orbit(focus, yaw, pitch, radius);
    info!("PanOrbit target transform: pos={:?}", t.translation);
    (t, focus)
}

fn transform_from_orbit(focus: Vec3, yaw: f32, pitch: f32, radius: f32) -> Transform {
    // gleiche Geometrie wie vorher, nur gekapselt
    let h = radius * pitch.cos();
    let x = h * yaw.sin();
    let z = h * yaw.cos();
    // Positive pitch hebt die Kamera über den Fokus
    let y = radius * pitch.sin();
    Transform::from_translation(focus + Vec3::new(x, y, z)).looking_at(focus, Vec3::Y)
}

fn compute_pan_orbit_transform_from(p: &PanOrbitCamera) -> Transform {
    // Nutze target_* (sind nach apply_pan_orbit_defaults gesetzt)
    let focus = p.target_focus;
    let yaw = p.target_yaw;
    let pitch = p.target_pitch;
    let radius = p.target_radius;
    transform_from_orbit(focus, yaw, pitch, radius)
}

fn apply_pan_orbit_defaults(
    pan_orbit: &mut PanOrbitCamera,
    focus: Vec3,
    defaults: &CameraDefaults,
) {
    let d = &defaults.pan_orbit;
    pan_orbit.focus = focus;
    pan_orbit.target_focus = focus;

    pan_orbit.radius = Some(d.radius);
    pan_orbit.target_radius = d.radius;

    pan_orbit.yaw = Some(d.yaw);
    pan_orbit.target_yaw = d.yaw;

    pan_orbit.pitch = Some(d.pitch);
    pan_orbit.target_pitch = d.pitch;

    // Limits und Sensitivitäten bleiben wie im Setup (könnten hier auch erneut gesetzt werden)
}
