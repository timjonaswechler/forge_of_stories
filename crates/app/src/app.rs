use chrono::{DateTime, Local};
use std::{fs, path::PathBuf};
use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct AppBase {
    pub app_id: &'static str,
    pub version: &'static str,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
    /// Guard to keep the non-blocking tracing worker alive (flushes log lines).
    pub log_guard: tracing_appender::non_blocking::WorkerGuard,
}

impl AppBase {
    pub fn app_id(&self) -> &'static str {
        self.app_id
    }
    pub fn version(&self) -> &'static str {
        self.version
    }
}

pub trait Application: Sized + 'static {
    type Error: From<BoxError> + std::fmt::Display + std::fmt::Debug + 'static;

    const APP_ID: &'static str;

    fn init_platform() -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn init<A: Application>(version: &'static str) -> Result<AppBase, A::Error> {
    let app_id = A::APP_ID;

    let config_dir = paths::config_dir().join(app_id);
    let data_dir = paths::data_dir().join(app_id);
    let logs_dir = paths::logs_dir().join(app_id);

    let current_local: DateTime<Local> = Local::now();
    let custom_format = current_local.format("%Y-%m-%dT%H:%M:%S").to_string();

    // Ensure directory structure exists
    fs::create_dir_all(&config_dir).ok();
    fs::create_dir_all(&data_dir).ok();
    fs::create_dir_all(&logs_dir).ok();

    let file_appender = tracing_appender::rolling::never(
        &logs_dir,
        paths::log_file(app_id, custom_format.as_str()),
    );
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    #[cfg(debug_assertions)]
    let level = LevelFilter::DEBUG; // Debug-Build zeigt DEBUG+INFO+...

    #[cfg(not(debug_assertions))]
    let level = LevelFilter::WARN; // Release-Build zeigt WARN+ERROR

    // Separate layer: file (non-blocking) + console (stdout)
    let file_layer = fmt::Layer::default()
        .with_target(false)
        .with_writer(non_blocking)
        .with_filter(level);

    let console_layer = fmt::Layer::default().with_target(false).with_filter(level);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    A::init_platform()?;

    Ok(AppBase {
        app_id,
        version,
        config_dir,
        data_dir,
        logs_dir,
        log_guard: guard,
    })
}
