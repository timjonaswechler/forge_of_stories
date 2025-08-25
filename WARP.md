# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Forge of Stories is a Rust game project built with Bevy 0.16 that supports both single-player and multiplayer modes. It uses a client-server architecture with QUIC networking protocol for encrypted communication across LAN and WAN environments.

## Architecture

This is a Rust workspace project with the following key crates:

### Core Crates
- **`wizard`** - TUI (Terminal User Interface) for server setup, management, and administration dashboard
- **`server`** - Dedicated server binary and library with Bevy game engine integration
- **`shared`** - Common networking code and messaging protocols shared between client and server
- **`settings`** - Centralized settings management with file watching and hot-reload capabilities  
- **`paths`** - Cross-platform path resolution for config, data, logs, and cache directories

### Network Architecture
- **Local Server**: Single-player mode with optional LAN/Steam Relay exposure
- **Dedicated Server**: Separate server binary for WAN deployment with admin dashboard
- **QUIC Protocol**: All network communication uses QUIC with encryption
- **TUI Dashboard**: Admin interface with optional web UI capability

### Settings System
The settings crate provides layered configuration management with:
- Profile-based settings with inheritance
- File watching for hot-reload
- Cross-process file locking for safe concurrent access
- TOML-based configuration with comment preservation

## Common Commands

### Build Commands
```bash
# Check all workspace crates
cargo check --workspace

# Build the entire workspace
cargo build --workspace

# Build release version
cargo build --workspace --release

# Build only the server
cargo build -p server

# Build only the wizard (TUI)
cargo build -p wizard
```

### Run Commands  
```bash
# Run the wizard (setup TUI)
cargo run -p wizard

# Run the dedicated server
cargo run -p server

# Run with specific tick/frame rates
cargo run -p wizard -- --tick-rate 60 --frame-rate 30
```

### Test Commands
```bash
# Run all tests
cargo test --workspace

# Test specific crate
cargo test -p settings
cargo test -p paths

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test -p paths test_name
```

### Development Commands
```bash
# Fix linting issues
cargo fix --workspace --allow-dirty

# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace

# Check for unused dependencies
cargo machete

# Generate documentation
cargo doc --workspace --open
```

## Development Workflow

### Working with Settings
The settings system is designed for extensibility:
- Settings are stored in platform-specific directories (see `paths` crate)
- TOML files preserve comments and formatting
- File watching enables hot-reload during development
- Use the layered configuration approach for defaults vs user overrides

### TUI Development
The wizard uses `ratatui` for the terminal interface:
- Page-based navigation system with action handlers
- Component-based UI architecture
- Keybinding system supports multi-key combinations
- Idle timeout automatically returns to login page

### Server Development
The server integrates Bevy ECS with networking:
- Bevy resources for cross-thread communication
- Channel-based messaging between TUI and game logic
- Certificate management for QUIC connections
- Stats collection for admin dashboard

### Networking
All network components use QUIC:
- Certificate generation handled automatically
- Encrypted connections by default
- Shared messaging protocols between client/server
- Support for both LAN and WAN deployment

## Key Directories

- `crates/network/wizard/` - Server admin TUI application
- `crates/network/server/` - Game server and dedicated server binary
- `crates/network/shared/` - Common networking protocols
- `crates/settings/` - Configuration management system
- `crates/paths/` - Platform-specific path resolution

## Important Files

- `Cargo.toml` - Workspace configuration with shared dependencies
- `crates/network/wizard/src/app.rs` - Main TUI application loop
- `crates/network/server/src/main.rs` - Server startup and Bevy integration
- `crates/settings/src/settings.rs` - Core settings management
- `crates/paths/src/paths.rs` - Platform path utilities

## Platform Support

Targets Windows, macOS, and Linux with platform-specific:
- Configuration directory resolution
- Data directory placement  
- Log file locations
- Cache directory handling

## Agent Guidelines

When working on this codebase:
- This is a game project with strict performance requirements
- The TUI should remain responsive during server operations
- Settings changes should be atomic and safely persisted
- Network code must handle connection drops gracefully
- Follow Rust editions 2024 patterns throughout
- Maintain separation between client/server/shared concerns
