//! Path context for runtime environment detection and project-aware paths.

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Identifies the runtime environment where the application is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEnvironment {
    /// Running via `cargo run` or in development mode
    Development,
    /// Running as an installed binary in production
    Production,
}

/// Context for managing application paths based on studio/project/app structure.
#[derive(Debug, Clone)]
pub struct PathContext {
    /// The runtime environment (development or production)
    environment: RuntimeEnvironment,
    /// Base path for all application data
    base_path: Arc<Path>,
    /// Studio identifier (e.g., "my_studio")
    studio: String,
    /// Project identifier (e.g., "my_game")
    project_id: String,
    /// Application identifier (e.g., "forge_of_stories")
    app_id: String,
}

impl PathContext {
    /// Creates a new PathContext with automatic environment detection.
    pub fn new(
        studio: impl Into<String>,
        project_id: impl Into<String>,
        app_id: impl Into<String>,
    ) -> Self {
        let environment = Self::detect_environment();
        let base_path = Self::determine_base_path(environment);

        Self {
            environment,
            base_path: base_path.into(),
            studio: studio.into(),
            project_id: project_id.into(),
            app_id: app_id.into(),
        }
    }

    /// Creates a PathContext with an explicit base path (useful for testing).
    pub fn with_base_path(
        base_path: PathBuf,
        studio: impl Into<String>,
        project_id: impl Into<String>,
        app_id: impl Into<String>,
    ) -> Self {
        let environment = Self::detect_environment();

        Self {
            environment,
            base_path: base_path.into(),
            studio: studio.into(),
            project_id: project_id.into(),
            app_id: app_id.into(),
        }
    }

    /// Detects the runtime environment based on executable location.
    fn detect_environment() -> RuntimeEnvironment {
        // Check if running from cargo (development)
        if let Ok(exe_path) = std::env::current_exe() {
            // If the executable is in a "target/debug" or "target/release" directory,
            // we're likely in development mode
            if exe_path.components().any(|c| c.as_os_str() == "target") {
                return RuntimeEnvironment::Development;
            }
        }

        // Check for cargo environment variables
        if std::env::var("CARGO").is_ok() || std::env::var("CARGO_MANIFEST_DIR").is_ok() {
            return RuntimeEnvironment::Development;
        }

        RuntimeEnvironment::Production
    }

    /// Determines the base path based on the runtime environment.
    fn determine_base_path(environment: RuntimeEnvironment) -> PathBuf {
        match environment {
            RuntimeEnvironment::Development => {
                // In development, use project root or current directory
                if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                    PathBuf::from(manifest_dir)
                } else if let Ok(current_dir) = std::env::current_dir() {
                    current_dir
                } else {
                    PathBuf::from(".")
                }
            }
            RuntimeEnvironment::Production => {
                // In production, use platform-specific data directory
                if cfg!(target_os = "macos") {
                    dirs::data_local_dir()
                        .expect("failed to determine Application Support directory")
                        .join("Forge_of_Stories")
                } else if cfg!(target_os = "windows") {
                    dirs::data_local_dir()
                        .expect("failed to determine LocalAppData directory")
                        .join("Forge_of_Stories")
                } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
                    dirs::data_local_dir()
                        .expect("failed to determine XDG_DATA_HOME directory")
                        .join("Forge_of_Stories")
                } else {
                    PathBuf::from(".")
                }
            }
        }
    }

    /// Returns the runtime environment.
    pub fn environment(&self) -> RuntimeEnvironment {
        self.environment
    }

    /// Returns the base path.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Returns the studio identifier.
    pub fn studio(&self) -> &str {
        &self.studio
    }

    /// Returns the project identifier.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Returns the app identifier.
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Returns the project root path: `<base>/<studio>/<project_id>`
    pub fn project_root(&self) -> PathBuf {
        self.base_path.join(&self.studio).join(&self.project_id)
    }

    /// Returns the settings file path: `<studio>/<project_id>/<app_id>.settings.json`
    pub fn settings_file(&self, app_id: Option<String>) -> PathBuf {
        self.project_root().join(format!(
            "{}.settings.json",
            app_id.unwrap_or(self.app_id.clone())
        ))
    }

    /// Returns the keybinding file path: `<studio>/<project_id>/keybinding.json`
    pub fn keybinding_file(&self) -> PathBuf {
        self.project_root().join("keybinding.json")
    }

    /// Returns the servers file path: `<studio>/<project_id>/<app_id>.servers.json`
    pub fn servers_file(&self) -> PathBuf {
        self.project_root()
            .join(format!("{}.servers.json", self.app_id))
    }

    /// Returns the versions directory path: `<studio>/<project_id>/versions/`
    pub fn versions_dir(&self) -> PathBuf {
        self.project_root().join("versions")
    }

    /// Returns a specific version path: `<studio>/<project_id>/versions/<app_id>.<version>`
    pub fn version_file(&self, version: &str) -> PathBuf {
        self.versions_dir()
            .join(format!("{}.{}", self.app_id, version))
    }

    /// Returns the saves directory path: `<studio>/<project_id>/saves/`
    pub fn saves_dir(&self) -> PathBuf {
        self.project_root().join("saves")
    }

    /// Returns a specific save directory: `<studio>/<project_id>/saves/<save_name>/`
    pub fn save_dir(&self, save_name: &str) -> PathBuf {
        self.saves_dir().join(save_name)
    }

    /// Returns the mods/dlcs directory path: `<studio>/<project_id>/mods/`
    pub fn mods_dir(&self) -> PathBuf {
        self.project_root().join("mods")
    }

    /// Returns the assets directory path: `<studio>/<project_id>/assets/<app_id>/`
    pub fn assets_dir(&self) -> PathBuf {
        self.project_root().join("assets").join(&self.app_id)
    }

    /// Returns the logs directory path: `<studio>/<project_id>/logs/`
    pub fn logs_dir(&self) -> PathBuf {
        self.project_root().join("logs")
    }

    /// Returns a log file path with timestamp: `<studio>/<project_id>/logs/<app_id>.<timestamp>.log`
    pub fn log_file(&self, timestamp: &str) -> PathBuf {
        self.logs_dir()
            .join(format!("{}.{}.log", self.app_id, timestamp))
    }

    /// Returns a log file path with current timestamp.
    pub fn log_file_now(&self) -> PathBuf {
        let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        self.log_file(&timestamp)
    }

    /// Ensures all necessary directories exist.
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        let dirs = vec![
            self.project_root(),
            self.versions_dir(),
            self.saves_dir(),
            self.mods_dir(),
            self.assets_dir(),
            self.logs_dir(),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_context_structure() {
        let ctx = PathContext::with_base_path(
            PathBuf::from("/test/base"),
            "my_studio",
            "my_project",
            "forge_of_stories",
        );

        assert_eq!(ctx.studio(), "my_studio");
        assert_eq!(ctx.project_id(), "my_project");
        assert_eq!(ctx.app_id(), "forge_of_stories");
        assert_eq!(
            ctx.project_root(),
            PathBuf::from("/test/base/my_studio/my_project")
        );
    }

    #[test]
    fn test_settings_paths() {
        let ctx = PathContext::with_base_path(PathBuf::from("/base"), "studio", "project", "app");

        assert_eq!(
            ctx.settings_file(None),
            PathBuf::from("/base/studio/project/app.settings.json")
        );
        assert_eq!(
            ctx.keybinding_file(),
            PathBuf::from("/base/studio/project/keybinding.json")
        );
        assert_eq!(
            ctx.servers_file(),
            PathBuf::from("/base/studio/project/app.servers.json")
        );
    }

    #[test]
    fn test_version_paths() {
        let ctx = PathContext::with_base_path(PathBuf::from("/base"), "studio", "project", "app");

        assert_eq!(
            ctx.version_file("1.0.0"),
            PathBuf::from("/base/studio/project/versions/app.1.0.0")
        );
    }

    #[test]
    fn test_save_paths() {
        let ctx = PathContext::with_base_path(PathBuf::from("/base"), "studio", "project", "app");

        assert_eq!(
            ctx.save_dir("save1"),
            PathBuf::from("/base/studio/project/saves/save1")
        );
    }

    #[test]
    fn test_log_file_path() {
        let ctx = PathContext::with_base_path(PathBuf::from("/base"), "studio", "project", "app");

        let log_path = ctx.log_file("20240315-120000");
        assert_eq!(
            log_path,
            PathBuf::from("/base/studio/project/logs/app.20240315-120000.log")
        );
    }
}
