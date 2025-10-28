use crate::GameState;
use crate::utils::{cleanup, remove};
use app::LOG_MAIN;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// Plugin for managing the splashscreen scene
pub struct SplashscreenScenePlugin;

impl Plugin for SplashscreenScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), setup_splashscreen)
            .add_systems(
                Update,
                (
                    animate_logo,
                    handle_skip_input,
                    auto_transition_to_main_menu,
                )
                    .run_if(in_state(GameState::Splashscreen)),
            )
            .add_systems(
                OnExit(GameState::Splashscreen),
                (
                    cleanup::<SplashscreenEntity>,
                    cleanup::<SplashscreenContext>,
                    remove::<SplashscreenTimer>,
                ),
            )
            .add_input_context::<SplashscreenContext>();
    }
}

/// Marker component for splashscreen entities
#[derive(Component)]
struct SplashscreenEntity;

/// Context for splashscreen input
#[derive(Component, Default)]
struct SplashscreenContext;

/// Skip splashscreen action
#[derive(InputAction)]
#[action_output(bool)]
struct SkipSplashscreen;

/// Timer for automatic transition to main menu
#[derive(Resource)]
struct SplashscreenTimer(Timer);

fn setup_splashscreen(mut commands: Commands) {
    info!(target: LOG_MAIN, "Setting up splashscreen scene");

    // Logo entity (visual)
    commands.spawn((
        Mesh3d(Handle::default()), // Will be replaced with actual mesh
        Transform::from_xyz(0.0, 0.0, 0.0),
        SplashscreenEntity,
        LogoAnimator {
            rotation_speed: 1.0,
        },
        Name::new("Logo Placeholder"),
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            SplashscreenEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SPLASHSCREEN"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));
        });

    // Input context entity (separate from visual entity)
    commands.spawn((
        Name::new("Splashscreen Input Context"),
        SplashscreenContext,
        actions!(
            SplashscreenContext[(
                Action::<SkipSplashscreen>::default(),
                bindings![KeyCode::Space, KeyCode::Enter, KeyCode::Escape],
            )]
        ),
    ));

    // Auto-transition timer
    commands.insert_resource(SplashscreenTimer(Timer::from_seconds(3.0, TimerMode::Once)));
}

/// Handle skip input
fn handle_skip_input(
    mut commands: Commands,
    actions: Query<(&ActionValue, &ActionOf<SplashscreenContext>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (value, _) in &actions {
        if value.as_bool() {
            info!(
                target: LOG_MAIN,
                "Splashscreen skipped by user input, transitioning to MainMenu"
            );
            next_state.set(GameState::MainMenu);
            commands.remove_resource::<SplashscreenTimer>();
            break;
        }
    }
}

/// Component for logo animation
#[derive(Component)]
struct LogoAnimator {
    rotation_speed: f32,
}

fn animate_logo(time: Res<Time>, mut query: Query<(&mut Transform, &LogoAnimator)>) {
    for (mut transform, animator) in &mut query {
        transform.rotate_y(animator.rotation_speed * time.delta_secs());
    }
}

fn auto_transition_to_main_menu(
    time: Res<Time>,
    mut timer: ResMut<SplashscreenTimer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        info!(target: LOG_MAIN, "Splashscreen finished, transitioning to MainMenu");
        next_state.set(GameState::MainMenu);
    }
}
