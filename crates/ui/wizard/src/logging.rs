use color_eyre::Result;
use paths::logs_dir;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

lazy_static::lazy_static! {
    pub static ref LOG_ENV: String = format!("FORGE_OF_STORIES_LOG_LEVEL");
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

pub fn init() -> Result<()> {
    let directory = logs_dir();
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join(LOG_FILE.clone());
    let log_file = std::fs::File::create(log_path)?;
    let env_filter = EnvFilter::builder().with_default_directive(tracing::Level::INFO.into());
    let env_filter = env_filter
        .try_from_env()
        .or_else(|_| env_filter.with_env_var(LOG_ENV.clone()).from_env())?;
    let file_subscriber = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(env_filter);
    let registry = tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default());

    // Bereits initialisiert? => ignorieren
    if let Err(_already_set) = registry.try_init() {
        // no-op
    }
    Ok(())
}
