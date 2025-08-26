# Paths crate: OS-specific directories, environment overrides, and directory creation

This crate centralizes all file system paths used by Forge_of_Stories for user data, configuration, logs, caches, prompts, extensions, and related files. It also provides helper functions to ensure required directories exist.

It is designed to be:
- OS-aware (macOS, Linux/FreeBSD, Windows)
- Overrideable via runtime and environment variables
- Safe and efficient (paths are computed once via a process-wide initializer)

## Resolution precedence

Unless explicitly noted, path resolution follows this order:

1) Runtime override
- `set_custom_data_dir("/some/path")` sets the base data directory for the entire process.
- When set, most locations derive from this directory.

2) Environment overrides (only if no runtime override is set)
- Data: `FORGE_OF_STORIES_DATA_DIR` or `FOS_DATA_DIR`
- Config: `FORGE_OF_STORIES_CONFIG_DIR` or `FOS_CONFIG_DIR`

3) OS defaults
- Platform conventions via the `dirs` crate (and Flatpak-aware envs on Linux/FreeBSD).
- Reasonable fallbacks for non-standard targets.

Notes:
- Environment variable values are used as provided. If you pass a relative path in an env var, it will be interpreted relative to the process’ current working directory.
- If you set a runtime override via `set_custom_data_dir`, config env vars can still override `config_dir()` independently.

## Key functions and their results

- data_dir() -> &'static PathBuf
  - 1) Runtime custom data directory (from `set_custom_data_dir`)
  - 2) Env: `FORGE_OF_STORIES_DATA_DIR` or `FOS_DATA_DIR`
  - 3) OS defaults:
    - macOS: `~/Library/Application Support/Forge_of_Stories`
    - Linux/FreeBSD: `$FLATPAK_XDG_DATA_HOME` if set; else `$XDG_DATA_HOME` via `dirs::data_local_dir`, then `/Forge_of_Stories`
    - Windows: `%LOCALAPPDATA%/Forge_of_Stories`
    - Other: falls back to `config_dir()` (see below)

- config_dir() -> &'static PathBuf
  - 1) Env: `FORGE_OF_STORIES_CONFIG_DIR` or `FOS_CONFIG_DIR`
  - 2) If runtime custom data directory is set: `<custom>/config`
  - 3) OS defaults:
    - Windows: `%APPDATA%/Forge_of_Stories` (Roaming AppData via `dirs::config_dir`)
    - Linux/FreeBSD: `$FLATPAK_XDG_CONFIG_HOME` if set; else `$XDG_CONFIG_HOME` via `dirs::config_dir`, then `/Forge_of_Stories`
    - Other (incl. macOS by default here): `~/.config/Forge_of_Stories`

- temp_dir() -> &'static PathBuf
  - macOS: `~/Library/Caches/Forge_of_Stories`
  - Windows: `%LOCALAPPDATA%/Forge_of_Stories` (via `dirs::cache_dir`)
  - Linux/FreeBSD: `$FLATPAK_XDG_CACHE_HOME` if set; else `$XDG_CACHE_HOME` via `dirs::cache_dir`, then `/Forge_of_Stories`
  - Other Unix: `~/.cache/Forge_of_Stories`

- logs_dir() -> &'static PathBuf
  - macOS: `~/Library/Logs/Forge_of_Stories`
  - Other platforms: `<data_dir>/logs`

- Log files
  - server_log_file(): `<logs_dir>/Forge_of_Stories.server.log`
  - client_log_file(): `<logs_dir>/Forge_of_Stories.client.log`
  - old_log_file(): `<logs_dir>/Forge_of_Stories.log.old`

- Settings and keymap
  - settings_file(): `<config_dir>/settings.toml`
  - global_settings_file(): `<config_dir>/global_settings.toml`
  - settings_backup_file(): `<config_dir>/settings_backup.toml`
  - keymap_file(): `<config_dir>/keymap.toml`
  - keymap_backup_file(): `<config_dir>/keymap_backup.toml`

- Other directories
  - extensions_dir(): `<data_dir>/extensions`
  - prompts_dir():
    - macOS → `<config_dir>/prompts`
    - other platforms → `<data_dir>/prompts`
  - prompt_overrides_dir(repo_path: Option<&Path>) -> PathBuf:
    - If `repo_path` is provided and `<repo>/assets/prompts` exists, returns that dev path.
    - Else macOS → `<config_dir>/prompt_overrides`, others → `<data_dir>/prompt_overrides`.
  - languages_dir(): `<data_dir>/languages`
  - home_dir(): platform home directory

## Environment variables recognized

- Base data directory
  - `FORGE_OF_STORIES_DATA_DIR`
  - `FOS_DATA_DIR` (alternative)

- Config directory
  - `FORGE_OF_STORIES_CONFIG_DIR`
  - `FOS_CONFIG_DIR` (alternative)

- Flatpak/XDG (Linux/FreeBSD)
  - `FLATPAK_XDG_DATA_HOME`, `FLATPAK_XDG_CONFIG_HOME`, `FLATPAK_XDG_CACHE_HOME`

Notes:
- If a runtime override was set via `set_custom_data_dir`, data env vars are ignored (config env vars still take precedence for the config dir).
- Use absolute paths in env vars when possible for clarity and portability.

## Directory creation helpers and behavior

- set_custom_data_dir(dir: &str)
  - Initializes the base data directory for the process.
  - If `dir` is a relative path, the implementation attempts to canonicalize it to an absolute path.
    - If canonicalization fails, this call will panic.
  - Ensures the directory exists by calling `std::fs::create_dir_all`.
  - Panics if called after `data_dir()` or `config_dir()` have already been initialized (the base must be set before first use).

- ensure_base_dirs() -> std::io::Result<()>
  - Ensures both `data_dir()` and `config_dir()` exist.
  - Internally uses `std::fs::create_dir_all` on each; if a directory already exists, `create_dir_all` is a no-op and returns `Ok(())`.
  - Returns the underlying `io::Error` on failure.

- ensure_logs_dir() -> std::io::Result<PathBuf>
  - Ensures `logs_dir()` exists (creates all missing parent components).
  - Returns the path to the ensured logs directory on success.
  - Returns the underlying `io::Error` on failure.

About create_dir_all:
- It creates all non-existing parent components.
- It is idempotent: if the directory already exists, it still returns `Ok(())`.
- On permission errors or invalid paths, it returns an `io::Error`.

## Platform examples (no env, no runtime override)

- macOS
  - data: `~/Library/Application Support/Forge_of_Stories`
  - config: `~/.config/Forge_of_Stories`
  - cache/temp: `~/Library/Caches/Forge_of_Stories`
  - logs: `~/Library/Logs/Forge_of_Stories`

- Linux/FreeBSD
  - data: `$XDG_DATA_HOME/Forge_of_Stories` (e.g., `~/.local/share/Forge_of_Stories`)
  - config: `$XDG_CONFIG_HOME/Forge_of_Stories` (e.g., `~/.config/Forge_of_Stories`)
  - cache/temp: `$XDG_CACHE_HOME/Forge_of_Stories` (e.g., `~/.cache/Forge_of_Stories`)
  - logs: `<data_dir>/logs`
  - If running under Flatpak and `FLATPAK_XDG_*` variables are present, those take precedence for data/config/cache.

- Windows
  - data: `%LOCALAPPDATA%/Forge_of_Stories`
  - config: `%APPDATA%/Forge_of_Stories`
  - cache/temp: `%LOCALAPPDATA%/Forge_of_Stories` (via `dirs::cache_dir`)
  - logs: `<data_dir>\logs`

## Examples (usage patterns)

Initialize custom base directory early in your program:
    // Must be called before first use of data_dir() or config_dir()
    paths::set_custom_data_dir("/srv/forge_of_stories");

Ensure base and logs directories exist:
    paths::ensure_base_dirs()?;
    let logs = paths::ensure_logs_dir()?;

Load settings file from the resolved config path:
    let settings_path = paths::settings_file();
    // open/read settings_path as needed

## Implementation notes

- All path getters return `&'static PathBuf` and are initialized exactly once per process with `OnceLock`.
- Path strings are sanitized appropriately (see `PathExt::to_sanitized_string` and `SanitizedPath` in the crate for details).
- macOS logs intentionally live under `~/Library/Logs/Forge_of_Stories` to adhere to platform conventions; on all other platforms, logs live under `<data_dir>/logs`.

If you need additional ensure_* helpers (e.g., for `extensions_dir()` or `prompts_dir()`), they can be added following the same pattern as `ensure_logs_dir()`.