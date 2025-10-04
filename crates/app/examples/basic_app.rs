//! Basic application example demonstrating app initialization with PathContext.
//!
//! This example shows how to create a simple application without Bevy,
//! using just the AppContext for path management and logging.

use app::{AppBuilder, Application, BoxError};

// Define your application
struct MyApp;

impl Application for MyApp {
    const APP_ID: &'static str = "my_app";
    // Uses defaults: STUDIO = "chicken105", PROJECT_ID = "forge_of_stories"
}

fn main() -> Result<(), BoxError> {
    println!("=== Basic App Example ===\n");

    // Initialize the application using the new builder pattern
    // This sets up paths, logging, and returns an AppContext
    let app_context = AppBuilder::<MyApp>::new("1.0.0")?.build_simple();

    println!("✅ Application initialized successfully!\n");

    // Access application info
    println!("📋 Application Info:");
    println!("   App ID: {}", app_context.app_id());
    println!("   Version: {}", app_context.version());
    println!();

    // Access PathContext for all path management
    let ctx = app_context.path_context();

    println!("📂 Path Structure:");
    println!("   Studio: {}", ctx.studio());
    println!("   Project: {}", ctx.project_id());
    println!("   Environment: {:?}", ctx.environment());
    println!("   Base Path: {:?}", ctx.base_path());
    println!();

    println!("📁 Directories:");
    println!("   Project Root: {:?}", ctx.project_root());
    println!("   Logs: {:?}", ctx.logs_dir());
    println!("   Saves: {:?}", ctx.saves_dir());
    println!("   Mods: {:?}", ctx.mods_dir());
    println!("   Assets: {:?}", ctx.assets_dir());
    println!();

    println!("📄 Configuration Files:");
    println!("   Settings: {:?}", ctx.settings_file(None));
    println!("   Keybindings: {:?}", ctx.keybinding_file());
    println!("   Servers: {:?}", ctx.servers_file());
    println!();

    println!("💡 Tip: Check the logs directory for the application log file!");
    println!("   Log file: {:?}", ctx.log_file_now());
    println!();

    println!("🎯 Note: This is a simple app without Bevy.");
    println!("   For Bevy apps, use `.build_with_bevy()` instead of `.build_simple()`");

    Ok(())
}
