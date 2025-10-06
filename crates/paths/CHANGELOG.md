# Changelog

All notable changes to the `paths` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-10-02

### Added
- **PathContext** - New context-based path management system
  - Runtime environment detection (Development vs Production)
  - Studio/Project/App hierarchical path structure
  - Automatic directory creation via `ensure_directories()`
  - Platform-aware base path resolution

- **Project-Aware Paths**
  - `settings_file()` - `<studio>/<project>/<app>.settings.json`
  - `keybinding_file()` - `<studio>/<project>/keybinding.json`
  - `servers_file()` - `<studio>/<project>/<app>.servers.json`
  - `versions_dir()` - `<studio>/<project>/versions/`
  - `version_file(version)` - `<studio>/<project>/versions/<app>.<version>`
  - `saves_dir()` - `<studio>/<project>/saves/`
  - `save_dir(name)` - `<studio>/<project>/saves/<name>/`
  - `mods_dir()` - `<studio>/<project>/mods/`
  - `assets_dir()` - `<studio>/<project>/assets/<app>/`
  - `logs_dir()` - `<studio>/<project>/logs/`
  - `log_file(timestamp)` - `<studio>/<project>/logs/<app>.<timestamp>.log`
  - `log_file_now()` - Automatically timestamped log file

- **Runtime Environment Detection**
  - `RuntimeEnvironment` enum (Development/Production)
  - Automatic detection based on executable location
  - CARGO environment variable detection
  - Platform-specific production paths

- **Examples**
  - `path_context.rs` - Basic PathContext usage
  - `integration.rs` - Real-world integration example with settings, saves, mods, etc.

- **Documentation**
  - Comprehensive README.md with usage examples
  - Detailed API documentation
  - Migration guide from v0.1.0

- **Dependencies**
  - Added `chrono` for timestamp generation

### Changed
- Bumped version from 0.1.0 to 0.2.0
- Enhanced module structure with `context` module

### Maintained
- Full backward compatibility with v0.1.0 API
- All legacy path functions (`config_dir()`, `data_dir()`, etc.)
- Environment variable overrides
- Platform-specific directory resolution
- PathExt trait and utilities

### Testing
- Added comprehensive unit tests for PathContext
- Tests for all path structure methods
- Environment detection validation
- Directory creation verification

## [0.1.0] - Initial Release

### Added
- Basic path resolution functions
  - `config_dir()` - Configuration directory
  - `data_dir()` - Data directory
  - `temp_dir()` - Temporary/cache directory
  - `logs_dir()` - Logs directory
  - `log_file(id, timestamp)` - Log file path
  - `extensions_dir()` - Extensions directory
  - `languages_dir()` - Languages directory
  - `home_dir()` - User home directory

- Platform-specific support
  - macOS (using Application Support, Library/Logs)
  - Linux/FreeBSD (XDG directories)
  - Windows (AppData directories)
  - Flatpak environment support

- Environment variable overrides
  - `FORGE_OF_STORIES_DATA_DIR` / `FOS_DATA_DIR`
  - `FORGE_OF_STORIES_CONFIG_DIR` / `FOS_CONFIG_DIR`

- PathExt trait
  - `compact()` - Compact path with ~ for home
  - `extension_or_hidden_file_name()` - Extension handling
  - `to_sanitized_string()` - Sanitized path strings

- Path utilities
  - `PathMatcher` - Glob pattern matching
  - `compare_paths()` - Path comparison
  - `NumericPrefixWithSuffix` - Numeric path sorting
  - `SanitizedPath` - Safe path handling

- Asset embedding support
  - `asset_str!()` macro for embedded assets

[0.2.0]: https://github.com/forge-of-stories/paths/releases/tag/v0.2.0
[0.1.0]: https://github.com/forge-of-stories/paths/releases/tag/v0.1.0