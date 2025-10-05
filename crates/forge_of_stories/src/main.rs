mod fos_app;
mod server;

use crate::fos_app::FOSApp;
use crate::server::{EmbeddedServer, ServerMode};
use app::AppBuilder;
use bevy::{color::palettes::basic::*, input_focus::InputFocus, log::LogPlugin, prelude::*};

/// Game state tracking where we are in the application flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
    InGame,
}

fn main() {
    let mut app = AppBuilder::<FOSApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|mut app, _ctx| {
            app.add_plugins(
                DefaultPlugins
                    .build()
                    .disable::<LogPlugin>()
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "Forge of Stories".to_string(),
                            ..default()
                        }),
                        ..default()
                    }),
            )
            .init_resource::<InputFocus>()
            .init_state::<GameState>()
            .add_systems(Startup, setup)
            .add_systems(Update, button_system.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnEnter(GameState::InGame), enter_game)
            .add_systems(
                FixedUpdate,
                crate::server::embedded::tick_embedded_server
                    .run_if(resource_exists::<EmbeddedServer>),
            );
            app
        });

    app.run();
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    // ui camera
    commands.spawn(Camera2d);
    commands.spawn(button());
}

/// System that runs when entering the InGame state
fn enter_game(mut commands: Commands, server: Res<EmbeddedServer>) {
    info!("Entered game state! Server mode: {:?}", server.mode());

    // Spawn a simple in-game UI showing we're in singleplayer
    commands.spawn((
        Text::new(format!("In Game - Server: {:?}", server.mode())),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn button_system(
    mut commands: Commands,
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                **text = "Starting...".to_string();
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);
                button.set_changed();

                // Start singleplayer game
                info!("Starting singleplayer game...");

                // Create embedded server with loopback mode
                match EmbeddedServer::new(ServerMode::Loopback) {
                    Ok(server) => {
                        info!("Embedded server started successfully");
                        commands.insert_resource(server);

                        // Transition to in-game state
                        next_state.set(GameState::InGame);
                    }
                    Err(e) => {
                        error!("Failed to start embedded server: {}", e);
                        **text = "Error!".to_string();
                    }
                }
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                **text = "Start Singleplayer".to_string();
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                **text = "Singleplayer".to_string();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn button() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BorderRadius::MAX,
            BackgroundColor(Color::BLACK),
            children![(
                Text::new("Singleplayer"),
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    )
}
