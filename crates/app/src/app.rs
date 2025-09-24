use chrono::{DateTime, Local};
use std::path::PathBuf;
use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct AppBase {
    pub app_id: &'static str,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppBase {
    pub fn app_id(&self) -> &'static str {
        self.app_id
    }
}

pub trait Application: Sized + 'static {
    type Error: From<BoxError> + std::fmt::Display + std::fmt::Debug + 'static;

    const APP_ID: &'static str;

    fn init_platform() -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn init<A: Application>() -> Result<AppBase, A::Error> {
    let app_id = A::APP_ID;

    let config_dir = paths::config_dir().join(app_id);
    let data_dir = paths::data_dir().join(app_id);
    let logs_dir = paths::logs_dir().join(app_id);

    let current_local: DateTime<Local> = Local::now();
    let custom_format = current_local.format("%Y-%m-%dT%H:%M:%S").to_string();

    let file_appender = tracing_appender::rolling::never(
        &logs_dir,
        paths::log_file(app_id, custom_format.as_str()),
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    #[cfg(debug_assertions)]
    let level = LevelFilter::DEBUG; // Debug-Build zeigt DEBUG+INFO+...

    #[cfg(not(debug_assertions))]
    let level = LevelFilter::WARN; // Release-Build zeigt WARN+ERROR

    tracing_subscriber::registry()
        .with(
            fmt::Layer::default()
                .with_writer(non_blocking)
                .with_filter(level),
        )
        .init();

    A::init_platform()?;

    Ok(AppBase {
        app_id,
        config_dir,
        data_dir,
        logs_dir,
    })
}
