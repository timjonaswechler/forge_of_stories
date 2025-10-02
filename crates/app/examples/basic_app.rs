//! Basic application example demonstrating app initialization with PathContext.

use app::{Application, BoxError};

// Define your application
struct MyApp;

impl Application for MyApp {
    type Error = BoxError;

    const APP_ID: &'static str = "my_app";
    // Uses defaults: STUDIO = "chicken105", PROJECT_ID = "forge_of_stories"

    fn init_platform() -> Result<(), Self::Error> {
        println!("🚀 Platform-specific initialization");
        Ok(())
    }
}

fn main() -> Result<(), BoxError> {
    println!("=== Basic App Example ===\n");

    // Initialize the application
    let app_base = app::init::<MyApp>("1.0.0")?;

    println!("✅ Application initialized successfully!\n");

    // Access application info
    println!("📋 Application Info:");
    println!("   App ID: {}", app_base.app_id());
    println!("   Version: {}", app_base.version());
    println!();

    // Access PathContext for all path management
    let ctx = app_base.path_context();

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

    Ok(())
}
