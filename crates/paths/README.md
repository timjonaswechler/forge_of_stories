# Paths Crate v0.2.0

Platform-specific path management for Forge of Stories with project-aware context and runtime environment detection.

## Overview

The `paths` crate provides a robust system for managing application paths with support for:

- **Runtime environment detection** (development vs. production)
- **Studio/Project/App hierarchical structure**
- **Platform-specific path resolution**
- **Automatic directory creation**
- **Timestamped log files**
- **Backward compatibility** with legacy path functions

## Quick Start

### Basic Usage

```rust
use paths::PathContext;

fn main() {
    // Create a path context for your project
    let ctx = PathContext::new(
        "my_studio",        // Studio name
        "awesome_game",     // Project ID
        "forge_of_stories", // App ID
    );

    // Get configuration file paths
    let settings = ctx.settings_file();
    let servers = ctx.servers_file();

    // Get directory paths
    let saves_dir = ctx.saves_dir();
    let mods_dir = ctx.mods_dir();
    let assets_dir = ctx.assets_dir();

    // Create a timestamped log file
    let log_file = ctx.log_file_now();

    // Ensure all directories exist
    ctx.ensure_directories().expect("Failed to create directories");
}
```

### Path Structure

The crate organizes paths in a hierarchical structure:

```
<base_path>/
└── <studio>/
    └── <project_id>/
        ├── <app_id>.settings.json
        ├── keybinding.json
        ├── data/
        ├── <app_id>.servers.json
        ├── versions/
        │   └── <app_id>.<version>
        ├── saves/
        │   └── <save_name>/
        ├── mods/
        ├── assets/
        │   └── <app_id>/
        └── logs/
            └── <app_id>.<timestamp>.log
```

## Runtime Environment Detection

The crate automatically detects whether it's running in development or production mode:

### Development Mode
- Detected when running via `cargo run`
- Uses project root directory as base path
- Checks for `CARGO_MANIFEST_DIR` or executable in `target/` directory

### Production Mode
- Detected when running as installed binary
- Uses platform-specific application data directory:
  - **macOS**: `~/Library/Application Support/Forge_of_Stories`
  - **Linux**: `~/.local/share/Forge_of_Stories`
  - **Windows**: `%LOCALAPPDATA%\Forge_of_Stories`

## PathContext API

### Creation

```rust
// Automatic environment detection
let ctx = PathContext::new("studio", "project", "app");

// Explicit base path (useful for testing)
let ctx = PathContext::with_base_path(
    PathBuf::from("/custom/path"),
    "studio",
    "project",
    "app"
);
```

### Configuration Files

```rust
ctx.settings_file()    // <studio>/<project>/app.settings.json
ctx.keybinding_file()  // <studio>/<project>/keybinding.json
ctx.servers_file()     // <studio>/<project>/app.servers.json
```

### Directories

```rust
ctx.project_root()  // <studio>/<project>
ctx.versions_dir()  // <studio>/<project>/versions
ctx.saves_dir()     // <studio>/<project>/saves
ctx.mods_dir()      // <studio>/<project>/mods
ctx.assets_dir()    // <studio>/<project>/assets/<app>
ctx.logs_dir()      // <studio>/<project>/logs
```

### Specific Paths

```rust
ctx.version_file("1.0.0")           // versions/app.1.0.0
ctx.save_dir("quicksave")           // saves/quicksave/
ctx.log_file("20240315-120000")     // logs/app.20240315-120000.log
ctx.log_file_now()                  // logs/app.<current_timestamp>.log
```

### Directory Management

```rust
// Create all necessary directories
ctx.ensure_directories()?;

// Check runtime environment
match ctx.environment() {
    RuntimeEnvironment::Development => {
        println!("Running in dev mode");
    }
    RuntimeEnvironment::Production => {
        println!("Running in production");
    }
}
```

## Legacy Path Functions

For backward compatibility, the crate still provides legacy path functions:

```rust
use paths::{config_dir, data_dir, temp_dir, logs_dir, log_file};

// Get standard directories (creates if doesn't exist)
let config = config_dir();
let data = data_dir();
let temp = temp_dir();
let logs = logs_dir();

// Get log file with custom ID and timestamp
let log = log_file("my_app", "20240315-120000");
```

### Environment Variables

Override default paths using environment variables:

- `FORGE_OF_STORIES_DATA_DIR` or `FOS_DATA_DIR` - Override data directory
- `FORGE_OF_STORIES_CONFIG_DIR` or `FOS_CONFIG_DIR` - Override config directory

## Path Utilities

### PathExt Trait

```rust
use paths::PathExt;

let path = PathBuf::from("/home/user/.config/app");
let compact = path.compact(); // ~/config/app (on Unix)
```

### Path Matching

```rust
use paths::PathMatcher;

let matcher = PathMatcher::new(&["*.json", "config/*"]).unwrap();
assert!(matcher.is_match(Path::new("config/settings.json")));
```

### Path Comparison

```rust
use paths::compare_paths;

let ordering = compare_paths(
    Path::new("a/b/c"),
    Path::new("a/b/d")
);
```

## Examples

Run the example to see PathContext in action:

```bash
cargo run -p paths --example path_context
```

## Testing

The crate includes comprehensive unit tests:

```bash
cargo test -p paths
```

## Integration with Settings Crate

The `paths` crate is designed to work seamlessly with the `settings` crate:

```rust
use paths::PathContext;
use settings::Settings;

let ctx = PathContext::new("studio", "project", "app");
let settings_path = ctx.settings_file();

// Load settings from PathContext-managed location
let settings = Settings::load_from_file(&settings_path)?;
```

## Migration from v0.1.0

If you're using legacy path functions, no changes are required. For new code, prefer `PathContext`:

**Before (v0.1.0):**
```rust
use paths::{data_dir, logs_dir};

let my_data = data_dir().join("my_app");
let my_logs = logs_dir().join("my_app.log");
```

**After (v0.2.0):**
```rust
use paths::PathContext;

let ctx = PathContext::new("studio", "project", "my_app");
let my_data = ctx.project_root();
let my_logs = ctx.log_file_now();
```

## Platform Support

- ✅ macOS
- ✅ Linux / FreeBSD
- ✅ Windows
- ✅ Flatpak (respects XDG environment variables)

## Version History

### v0.2.0
- Added `PathContext` for project-aware path management
- Runtime environment detection (dev/production)
- Studio/Project/App hierarchical structure
- Timestamped log file support
- Automatic directory creation
- Comprehensive examples and tests

### v0.1.0
- Initial release with basic path functions
- Platform-specific directory resolution
- Environment variable overrides
