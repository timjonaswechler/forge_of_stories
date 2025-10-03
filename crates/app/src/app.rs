use paths::PathContext;
#[cfg(debug_assertions)]
use std::hash::BuildHasherDefault;
use std::path::PathBuf;
use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct AppBase {
    pub path_context: PathContext,
    pub version: &'static str,
    pub log_guard: tracing_appender::non_blocking::WorkerGuard,
}

impl AppBase {
    pub fn app_id(&self) -> &str {
        self.path_context.app_id()
    }

    pub fn version(&self) -> &'static str {
        self.version
    }

    pub fn path_context(&self) -> &PathContext {
        &self.path_context
    }
}

pub trait Application: Sized + 'static {
    type Error: From<BoxError> + std::fmt::Display + std::fmt::Debug + 'static;

    const APP_ID: &'static str;
    const STUDIO: &'static str = "chicken105";
    const PROJECT_ID: &'static str = "forge_of_stories";

    fn init_platform() -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn init<A: Application>(version: &'static str) -> Result<AppBase, A::Error> {
    let app_id = A::APP_ID;
    let studio = A::STUDIO;
    let project_id = A::PROJECT_ID;

    // Create PathContext with studio/project/app hierarchy
    #[cfg(debug_assertions)]
    let path_context = PathContext::with_base_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(".out"),
        studio,
        project_id,
        app_id,
    );
    #[cfg(not(debug_assertions))]
    let path_context = PathContext::new(studio, project_id, app_id);

    // Ensure all directories exist
    path_context.ensure_directories().map_err(BoxError::from)?;

    // Get log file path and split into directory + filename
    let log_file_path = path_context.log_file_now();
    let log_dir = log_file_path
        .parent()
        .expect("log file path should have parent directory");
    let log_filename = log_file_path
        .file_name()
        .expect("log file path should have filename");

    let file_appender = tracing_appender::rolling::never(log_dir, log_filename);
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
        path_context,
        version,
        log_guard: guard,
    })
}
