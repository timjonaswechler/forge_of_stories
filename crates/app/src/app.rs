use paths::PathContext;
use std::fs::File;
use std::marker::PhantomData;
#[cfg(debug_assertions)]
use std::path::PathBuf;
use tracing_subscriber::{
    Layer, filter, filter::LevelFilter, filter::filter_fn, fmt, layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Application infrastructure context.
///
/// Contains path management, version info, and logging infrastructure.
/// This is the core context that every application needs, regardless of
/// whether it uses Bevy or not.
pub struct AppContext {
    pub path_context: PathContext,
    pub version: &'static str,
    /// The log guard must be kept alive for the duration of the application
    /// to ensure log messages are properly flushed.
    _log_guard: tracing_appender::non_blocking::WorkerGuard,
}

impl AppContext {
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

/// Application metadata trait.
///
/// Define your application's identity by implementing this trait.
/// This is a pure marker trait - no logic, just constants.
pub trait Application: Sized + 'static {
    const APP_ID: &'static str;
    const STUDIO: &'static str = "chicken105";
    const PROJECT_ID: &'static str = "forge_of_stories";
}

/// Builder for creating applications with proper initialization.
///
/// Use this to create either simple applications (just `AppContext`)
/// or Bevy-based applications (`BevyApp<A>`).
pub struct AppBuilder<A: Application> {
    context: AppContext,
    _marker: PhantomData<A>,
}

impl<A: Application> AppBuilder<A> {
    /// Create a new application builder.
    ///
    /// This performs all the common initialization:
    /// - Sets up path context (platform-specific directories)
    /// - Initializes logging (file + console)
    /// - Ensures all directories exist
    pub fn new(version: &'static str) -> Result<Self, BoxError> {
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
        path_context.ensure_directories()?;

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
        let level = LevelFilter::INFO;

        #[cfg(not(debug_assertions))]
        let level = LevelFilter::WARN;

        // Separate layer: file (non-blocking) + console (stdout)
        let file_layer = fmt::Layer::default()
            .with_target(false)
            .with_ansi(false)
            .with_writer(non_blocking)
            .with_filter(filter_fn(move |metadata| metadata.level() <= &level));

        let console_layer = fmt::Layer::default()
            .with_target(false)
            .with_filter(filter_fn(move |metadata| metadata.level() <= &level));

        tracing_subscriber::registry()
            .with(file_layer)
            .with(console_layer)
            .init();

        Ok(Self {
            context: AppContext {
                path_context,
                version,
                _log_guard: guard,
            },
            _marker: PhantomData,
        })
    }

    /// Build a simple application (no Bevy).
    ///
    /// Returns just the `AppContext` for applications that don't need Bevy.
    pub fn build_simple(self) -> AppContext {
        self.context
    }

    /// Build a Bevy-based application.
    ///
    /// The `configure` callback receives the Bevy `App` by value and the `AppContext`,
    /// and must return the configured App. This follows Bevy's builder pattern.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let app = AppBuilder::<MyApp>::new("1.0.0")?
    ///     .build_with_bevy(|app, ctx| {
    ///         app.add_plugins(DefaultPlugins)
    ///            .add_systems(Update, my_system)
    ///     });
    /// ```
    #[cfg(feature = "bevy")]
    pub fn build_with_bevy(
        self,
        configure: impl FnOnce(bevy::prelude::App, &AppContext) -> bevy::prelude::App,
    ) -> BevyApp<A> {
        let bevy_app = bevy::prelude::App::new();
        let configured_app = configure(bevy_app, &self.context);

        BevyApp {
            context: self.context,
            app: configured_app,
            _marker: PhantomData,
        }
    }
}

/// Bevy-based application wrapper.
///
/// Contains both the infrastructure context and the Bevy App.
/// The context is kept alive to ensure logging continues working.
#[cfg(feature = "bevy")]
pub struct BevyApp<A: Application> {
    pub context: AppContext,
    pub app: bevy::prelude::App,
    _marker: PhantomData<A>,
}

#[cfg(feature = "bevy")]
impl<A: Application> BevyApp<A> {
    /// Run the Bevy application.
    pub fn run(&mut self) {
        self.app.run();
    }

    /// Get a reference to the app context.
    pub fn context(&self) -> &AppContext {
        &self.context
    }

    /// Get a mutable reference to the Bevy app.
    pub fn app_mut(&mut self) -> &mut bevy::prelude::App {
        &mut self.app
    }
}

// ============================================================================
// Legacy compatibility layer
// ============================================================================

/// Legacy alias for `AppContext`.
///
/// **Deprecated**: Use `AppContext` directly instead.
#[deprecated(since = "0.2.0", note = "Use AppContext instead")]
pub type AppBase = AppContext;

/// Legacy initialization function.
///
/// **Deprecated**: Use `AppBuilder::new()` instead.
#[deprecated(since = "0.2.0", note = "Use AppBuilder::new().build_simple() instead")]
pub fn init<A: Application>(version: &'static str) -> Result<AppContext, BoxError> {
    AppBuilder::<A>::new(version).map(|builder| builder.build_simple())
}
