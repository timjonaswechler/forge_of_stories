//! Paths to locations used by Forge_of_Stories.

use globset::{Glob, GlobSet, GlobSetBuilder};

use std::cmp::{self, Ordering};
use std::ffi::OsStr;
use std::path::{Path, PathBuf, StripPrefixError};
use std::sync::{Arc, OnceLock};
use unicase::UniCase;

static CUSTOM_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

static CUSTOM_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

static CURRENT_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

static CURRENT_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

const ENV_DATA_DIR: &str = "FORGE_OF_STORIES_DATA_DIR";
const ENV_CONFIG_DIR: &str = "FORGE_OF_STORIES_CONFIG_DIR";
const ENV_DATA_DIR_ALT: &str = "FOS_DATA_DIR";
const ENV_CONFIG_DIR_ALT: &str = "FOS_CONFIG_DIR";

/// Sets a custom directory for configuration data, overriding the default config directory.
/// This function must be called before any other path operations that depend on the config directory.
/// The directory's path will be canonicaliForge_of_Stories to an absolute path by a blocking FS operation.
/// The directory will be created if it doesn't exist.
///
/// # Arguments
///
/// * `dir` - The path to use as the custom config directory.
///
/// # Returns
///
/// A reference to the static `PathBuf` containing the custom config directory path.
///
/// # Panics
///
/// Panics if:
/// * Called after the config directory has been initialiForge_of_Stories (e.g., via `config_dir`)
/// * The directory's path cannot be canonicaliForge_of_Stories to an absolute path
/// * The directory cannot be created
pub fn set_custom_config_dir(dir: &str) -> &'static PathBuf {
    if CURRENT_DATA_DIR.get().is_some() || CURRENT_CONFIG_DIR.get().is_some() {
        panic!("set_custom_config_dir called after data_dir or config_dir was initialized");
    }
    CUSTOM_CONFIG_DIR.get_or_init(|| {
        let mut path = PathBuf::from(dir);
        if path.is_relative() {
            let abs_path = path.canonicalize().expect(
                "failed to canonicalize custom config directory's path to an absolute path",
            );
            path = PathBuf::from(SanitizedPath::from(abs_path))
        }
        std::fs::create_dir_all(&path).expect("failed to create custom config directory");
        path
    })
}

/// Returns the path to the configuration directory used by Forge_of_Stories.
pub fn config_dir() -> &'static PathBuf {
    CURRENT_CONFIG_DIR.get_or_init(|| {
        // 1) ENV overrides
        if let Ok(p) = std::env::var(ENV_CONFIG_DIR).or_else(|_| std::env::var(ENV_CONFIG_DIR_ALT))
        {
            PathBuf::from(p)
        } else if let Some(custom_config) = CUSTOM_CONFIG_DIR.get() {
            // 2) Runtime override via custom config dir
            custom_config.clone()
        } else if let Some(custom_dir) = CUSTOM_DATA_DIR.get() {
            // 3) Runtime override via custom data dir
            custom_dir.join("config")
        } else if cfg!(target_os = "windows") {
            // 4) OS defaults
            dirs::config_dir()
                .expect("failed to determine RoamingAppData directory")
                .join("Forge_of_Stories")
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_config) = std::env::var("FLATPAK_XDG_CONFIG_HOME") {
                flatpak_xdg_config.into()
            } else {
                dirs::config_dir().expect("failed to determine XDG_CONFIG_HOME directory")
            }
            .join("Forge_of_Stories")
        } else {
            home_dir().join(".config").join("Forge_of_Stories")
        }
    })
}

/// Returns the path to the data directory used by Forge_of_Stories.
pub fn data_dir() -> &'static PathBuf {
    CURRENT_DATA_DIR.get_or_init(|| {
        // 1) Runtime override via custom data dir
        if let Some(custom_dir) = CUSTOM_DATA_DIR.get() {
            custom_dir.clone()
        // 2) ENV overrides
        } else if let Ok(p) =
            std::env::var(ENV_DATA_DIR).or_else(|_| std::env::var(ENV_DATA_DIR_ALT))
        {
            PathBuf::from(p)
        // 3) OS defaults
        } else if cfg!(target_os = "macos") {
            home_dir().join("Library/Application Support/Forge_of_Stories")
        } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            if let Ok(flatpak_xdg_data) = std::env::var("FLATPAK_XDG_DATA_HOME") {
                flatpak_xdg_data.into()
            } else {
                dirs::data_local_dir().expect("failed to determine XDG_DATA_HOME directory")
            }
            .join("Forge_of_Stories")
        } else if cfg!(target_os = "windows") {
            dirs::data_local_dir()
                .expect("failed to determine LocalAppData directory")
                .join("Forge_of_Stories")
        } else {
            config_dir().clone() // Fallback
        }
    })
}

/// Returns the path to the temp directory used by Forge_of_Stories.
pub fn temp_dir() -> &'static PathBuf {
    static TEMP_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEMP_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            return dirs::cache_dir()
                .expect("failed to determine cachesDirectory directory")
                .join("Forge_of_Stories");
        }

        if cfg!(target_os = "windows") {
            return dirs::cache_dir()
                .expect("failed to determine LocalAppData directory")
                .join("Forge_of_Stories");
        }

        if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            return if let Ok(flatpak_xdg_cache) = std::env::var("FLATPAK_XDG_CACHE_HOME") {
                flatpak_xdg_cache.into()
            } else {
                dirs::cache_dir().expect("failed to determine XDG_CACHE_HOME directory")
            }
            .join("Forge_of_Stories");
        }

        home_dir().join(".cache").join("Forge_of_Stories")
    })
}

/// Returns the path to the logs directory.
pub fn logs_dir() -> &'static PathBuf {
    static LOGS_DIR: OnceLock<PathBuf> = OnceLock::new();
    LOGS_DIR.get_or_init(|| {
        if cfg!(target_os = "macos") {
            home_dir().join("Library/Logs/Forge_of_Stories")
        } else {
            data_dir().join("logs")
        }
    })
}

/// Ensures that the base data and config directories exist.
pub fn ensure_base_dirs() -> std::io::Result<()> {
    let dd = data_dir().clone();
    let cd = config_dir().clone();
    if !dd.exists() {
        std::fs::create_dir_all(&dd)?;
    }
    if !cd.exists() {
        std::fs::create_dir_all(&cd)?;
    }
    Ok(())
}

/// Ensures that the logs directory exists and returns it.
pub fn ensure_logs_dir() -> std::io::Result<PathBuf> {
    let ld = logs_dir().clone();
    if !ld.exists() {
        std::fs::create_dir_all(&ld)?;
    }
    Ok(ld)
}

/// Returns the path to the extensions directory.
///
/// This is where installed extensions are stored.
pub fn extensions_dir() -> &'static PathBuf {
    static EXTENSIONS_DIR: OnceLock<PathBuf> = OnceLock::new();
    EXTENSIONS_DIR.get_or_init(|| data_dir().join("extensions"))
}

/// Returns the path to the languages directory.
///
/// This is where language servers are downloaded to for languages built-in to Forge_of_Stories.
pub fn languages_dir() -> &'static PathBuf {
    static LANGUAGES_DIR: OnceLock<PathBuf> = OnceLock::new();
    LANGUAGES_DIR.get_or_init(|| data_dir().join("languages"))
}

/// Returns the path to the user's home directory.
pub fn home_dir() -> &'static PathBuf {
    static HOME_DIR: OnceLock<PathBuf> = OnceLock::new();
    HOME_DIR.get_or_init(|| dirs::home_dir().expect("failed to determine home directory"))
}

pub trait PathExt {
    fn compact(&self) -> PathBuf;
    fn extension_or_hidden_file_name(&self) -> Option<&str>;
    fn to_sanitized_string(&self) -> String;
    fn try_from_bytes<'a>(bytes: &'a [u8]) -> anyhow::Result<Self>
    where
        Self: From<&'a Path>,
    {
        #[cfg(unix)]
        {
            use std::os::unix::prelude::OsStrExt;
            Ok(Self::from(Path::new(OsStr::from_bytes(bytes))))
        }
        #[cfg(windows)]
        {
            use anyhow::Context as _;
            use tendril::fmt::{Format, WTF8};
            WTF8::validate(bytes)
                .then(|| {
                    // Safety: bytes are valid WTF-8 sequence.
                    Self::from(Path::new(unsafe {
                        OsStr::from_encoded_bytes_unchecked(bytes)
                    }))
                })
                .with_context(|| format!("Invalid WTF-8 sequence: {bytes:?}"))
        }
    }
}

impl<T: AsRef<Path>> PathExt for T {
    /// Compacts a given file path by replacing the user's home directory
    /// prefix with a tilde (`~`).
    ///
    /// # Returns
    ///
    /// * A `PathBuf` containing the compacted file path. If the input path
    ///   does not have the user's home directory prefix, or if we are not on
    ///   Linux or macOS, the original path is returned unchanged.
    fn compact(&self) -> PathBuf {
        if cfg!(any(target_os = "linux", target_os = "freebsd")) || cfg!(target_os = "macos") {
            match self.as_ref().strip_prefix(home_dir().as_path()) {
                Ok(relative_path) => {
                    let mut shortened_path = PathBuf::new();
                    shortened_path.push("~");
                    shortened_path.push(relative_path);
                    shortened_path
                }
                Err(_) => self.as_ref().to_path_buf(),
            }
        } else {
            self.as_ref().to_path_buf()
        }
    }

    /// Returns a file's extension or, if the file is hidden, its name without the leading dot
    fn extension_or_hidden_file_name(&self) -> Option<&str> {
        let path = self.as_ref();
        let file_name = path.file_name()?.to_str()?;
        if file_name.starts_with('.') {
            return file_name.strip_prefix('.');
        }

        path.extension()
            .and_then(|e| e.to_str())
            .or_else(|| path.file_stem()?.to_str())
    }

    /// Returns a sanitized string representation of the path.
    /// Note, on Windows, this assumes that the path is a valid UTF-8 string and
    /// is not a UNC path.
    fn to_sanitized_string(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            self.as_ref().to_string_lossy().replace("/", "\\")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.as_ref().to_string_lossy().to_string()
        }
    }
}

/// Due to the issue of UNC paths on Windows, which can cause bugs in various parts of Zed, introducing this `SanitizedPath`
/// leverages Rust's type system to ensure that all paths entering Zed are always "sanitized" by removing the `\\\\?\\` prefix.
/// On non-Windows operating systems, this struct is effectively a no-op.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SanitizedPath(pub Arc<Path>);

impl SanitizedPath {
    pub fn starts_with(&self, prefix: &SanitizedPath) -> bool {
        self.0.starts_with(&prefix.0)
    }

    pub fn as_path(&self) -> &Arc<Path> {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.to_string_lossy().to_string()
    }

    pub fn to_glob_string(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            self.0.to_string_lossy().replace("/", "\\")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.0.to_string_lossy().to_string()
        }
    }

    pub fn join(&self, path: &Self) -> Self {
        self.0.join(&path.0).into()
    }

    pub fn strip_prefix(&self, base: &Self) -> Result<&Path, StripPrefixError> {
        self.0.strip_prefix(base.as_path())
    }
}

impl From<SanitizedPath> for Arc<Path> {
    fn from(sanitized_path: SanitizedPath) -> Self {
        sanitized_path.0
    }
}

impl From<SanitizedPath> for PathBuf {
    fn from(sanitized_path: SanitizedPath) -> Self {
        sanitized_path.0.as_ref().into()
    }
}

impl<T: AsRef<Path>> From<T> for SanitizedPath {
    #[cfg(not(target_os = "windows"))]
    fn from(path: T) -> Self {
        let path = path.as_ref();
        SanitizedPath(path.into())
    }

    #[cfg(target_os = "windows")]
    fn from(path: T) -> Self {
        let path = path.as_ref();
        SanitizedPath(dunce::simplified(path).into())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathStyle {
    Posix,
    Windows,
}

impl PathStyle {
    #[cfg(target_os = "windows")]
    pub const fn current() -> Self {
        PathStyle::Windows
    }

    #[cfg(not(target_os = "windows"))]
    pub const fn current() -> Self {
        PathStyle::Posix
    }

    #[inline]
    pub fn separator(&self) -> &str {
        match self {
            PathStyle::Posix => "/",
            PathStyle::Windows => "\\",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PathMatcher {
    sources: Vec<String>,
    glob: GlobSet,
}

// impl std::fmt::Display for PathMatcher {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.sources.fmt(f)
//     }
// }

impl PartialEq for PathMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.sources.eq(&other.sources)
    }
}

impl Eq for PathMatcher {}

impl PathMatcher {
    pub fn new(globs: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Self, globset::Error> {
        let globs = globs
            .into_iter()
            .map(|as_str| Glob::new(as_str.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        let sources = globs.iter().map(|glob| glob.glob().to_owned()).collect();
        let mut glob_builder = GlobSetBuilder::new();
        for single_glob in globs {
            glob_builder.add(single_glob);
        }
        let glob = glob_builder.build()?;
        Ok(PathMatcher { glob, sources })
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn is_match<P: AsRef<Path>>(&self, other: P) -> bool {
        let other_path = other.as_ref();
        self.sources.iter().any(|source| {
            let as_bytes = other_path.as_os_str().as_encoded_bytes();
            as_bytes.starts_with(source.as_bytes()) || as_bytes.ends_with(source.as_bytes())
        }) || self.glob.is_match(other_path)
            || self.check_with_end_separator(other_path)
    }

    fn check_with_end_separator(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let separator = std::path::MAIN_SEPARATOR_STR;
        if path_str.ends_with(separator) {
            false
        } else {
            self.glob.is_match(path_str.to_string() + separator)
        }
    }
}

pub fn compare_paths(
    (path_a, a_is_file): (&Path, bool),
    (path_b, b_is_file): (&Path, bool),
) -> cmp::Ordering {
    let mut components_a = path_a.components().peekable();
    let mut components_b = path_b.components().peekable();
    loop {
        match (components_a.next(), components_b.next()) {
            (Some(component_a), Some(component_b)) => {
                let a_is_file = components_a.peek().is_none() && a_is_file;
                let b_is_file = components_b.peek().is_none() && b_is_file;
                let ordering = a_is_file.cmp(&b_is_file).then_with(|| {
                    let path_a = Path::new(component_a.as_os_str());
                    let path_string_a = if a_is_file {
                        path_a.file_stem()
                    } else {
                        path_a.file_name()
                    }
                    .map(|s| s.to_string_lossy());
                    let num_and_remainder_a = path_string_a
                        .as_deref()
                        .map(NumericPrefixWithSuffix::from_numeric_prefixed_str);

                    let path_b = Path::new(component_b.as_os_str());
                    let path_string_b = if b_is_file {
                        path_b.file_stem()
                    } else {
                        path_b.file_name()
                    }
                    .map(|s| s.to_string_lossy());
                    let num_and_remainder_b = path_string_b
                        .as_deref()
                        .map(NumericPrefixWithSuffix::from_numeric_prefixed_str);

                    num_and_remainder_a.cmp(&num_and_remainder_b).then_with(|| {
                        if a_is_file && b_is_file {
                            let ext_a = path_a.extension().unwrap_or_default();
                            let ext_b = path_b.extension().unwrap_or_default();
                            ext_a.cmp(ext_b)
                        } else {
                            cmp::Ordering::Equal
                        }
                    })
                });
                if !ordering.is_eq() {
                    return ordering;
                }
            }
            (Some(_), None) => break cmp::Ordering::Greater,
            (None, Some(_)) => break cmp::Ordering::Less,
            (None, None) => break cmp::Ordering::Equal,
        }
    }
}

/// A way to sort strings with starting numbers numerically first, falling back to alphanumeric one,
/// case-insensitive.
///
/// This is useful for turning regular alphanumerically sorted sequences as `1-abc, 10, 11-def, .., 2, 21-abc`
/// into `1-abc, 2, 10, 11-def, .., 21-abc`
#[derive(Debug, PartialEq, Eq)]
pub struct NumericPrefixWithSuffix<'a>(Option<u64>, &'a str);

impl<'a> NumericPrefixWithSuffix<'a> {
    pub fn from_numeric_prefixed_str(str: &'a str) -> Self {
        let i = str.chars().take_while(|c| c.is_ascii_digit()).count();
        let (prefix, remainder) = str.split_at(i);

        let prefix = prefix.parse().ok();
        Self(prefix, remainder)
    }
}

/// When dealing with equality, we need to consider the case of the strings to achieve strict equality
/// to handle cases like "a" < "A" instead of "a" == "A".
impl Ord for NumericPrefixWithSuffix<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0, other.0) {
            (None, None) => UniCase::new(self.1)
                .cmp(&UniCase::new(other.1))
                .then_with(|| self.1.cmp(other.1).reverse()),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(&b).then_with(|| {
                UniCase::new(self.1)
                    .cmp(&UniCase::new(other.1))
                    .then_with(|| self.1.cmp(other.1).reverse())
            }),
        }
    }
}

impl PartialOrd for NumericPrefixWithSuffix<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
