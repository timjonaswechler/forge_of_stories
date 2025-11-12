use bevy::{log::LogPlugin, prelude::*};
use bevy_paths::{PathMarker, PathRegistry, PathRegistryPlugin};
use bevy_settings::{SerializationFormat, SettingsPlugin};
use game_server::settings::Network;
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::filter_fn, fmt, prelude::*};

const APP_ID: &'static str = "forge_of_stories";
const STUDIO: &'static str = "chicken105";
const PROJECT_ID: &'static str = "forge_of_stories";

#[derive(PathMarker, Resource)]
pub struct LogsDir;

#[derive(PathMarker, Resource)]
pub struct SettingsDir;

pub fn init() -> App {
    // Create PathContext with studio/project/app hierarchy
    let mut paths_plugin = PathRegistryPlugin::new(STUDIO, PROJECT_ID, APP_ID);
    #[cfg(debug_assertions)]
    {
        paths_plugin = paths_plugin.with_base_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join(".out"),
        );
    }
    paths_plugin = paths_plugin
        .register::<LogsDir>("logs/")
        .expect("Failed to register LogsDir");
    paths_plugin = paths_plugin
        .register::<SettingsDir>("settings/")
        .expect("Failed to register SettingsDir");

    let mut app = App::new();
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
    );
    app.add_plugins(paths_plugin);

    let registry = app.world().resource::<PathRegistry>();
    let settings_file = registry.get::<SettingsDir>().unwrap().join("settings.json");

    app.add_plugins(
        SettingsPlugin::new()
            .with_path(settings_file, SerializationFormat::Json)
            .register::<Network>(),
    );
    app.add_systems(Startup, setup_logging);
    println!("log");
    app
}

fn setup_logging(world: &mut World) {
    // Logging
    let registry = world.resource::<PathRegistry>();
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let log_file = registry
        .get::<LogsDir>()
        .unwrap()
        .join(format!("{}.{}.log", APP_ID, timestamp));

    // Get log file path and split into directory + filename
    let log_dir = log_file
        .parent()
        .expect("log file path should have parent directory");
    let log_filename = log_file
        .file_name()
        .expect("log file path should have filename");

    let file_appender = tracing_appender::rolling::never(log_dir, log_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    #[cfg(debug_assertions)]
    let level = LevelFilter::INFO;

    #[cfg(not(debug_assertions))]
    let level = LevelFilter::WARN;

    // Separate layer: file (non-blocking) + console (stdout)
    let file_layer = fmt::Layer::default()
        .with_target(true)
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(filter_fn(move |metadata| metadata.level() <= &level));

    let console_layer = fmt::Layer::default()
        .with_target(true)
        .with_filter(filter_fn(move |metadata| metadata.level() <= &level));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();
}

pub const LOG_MAIN: &str = "main";
pub const LOG_CLIENT_HOST: &str = "client/host";
pub const LOG_CLIENT_APP: &str = "client/app";
pub const LOG_CLIENT: &str = "client";
