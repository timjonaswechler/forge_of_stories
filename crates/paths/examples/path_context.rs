//! Example demonstrating the usage of PathContext for project-aware paths.

use paths::{PathContext, RuntimeEnvironment};

fn main() {
    println!("=== PathContext Example ===\n");

    // Create a path context for your project
    let ctx = PathContext::new(
        "my_studio",        // Studio name
        "awesome_game",     // Project ID
        "forge_of_stories", // App ID
    );

    // Display environment info
    println!("Runtime Environment: {:?}", ctx.environment());
    println!("Base Path: {:?}", ctx.base_path());
    println!("Studio: {}", ctx.studio());
    println!("Project ID: {}", ctx.project_id());
    println!("App ID: {}\n", ctx.app_id());

    // Display project structure
    println!("=== Project Structure ===");
    println!("Project Root: {:?}", ctx.project_root());
    println!();

    // Configuration files
    println!("=== Configuration Files ===");
    println!("Settings: {:?}", ctx.settings_file(None));
    println!("Keybindings: {:?}", ctx.keybinding_file());
    println!("Servers: {:?}", ctx.servers_file());
    println!();

    // Directories
    println!("=== Directories ===");
    println!("Versions: {:?}", ctx.versions_dir());
    println!("Saves: {:?}", ctx.saves_dir());
    println!("Mods: {:?}", ctx.mods_dir());
    println!("Assets: {:?}", ctx.assets_dir());
    println!("Logs: {:?}", ctx.logs_dir());
    println!();

    // Specific paths
    println!("=== Specific Paths ===");
    println!("Version 1.0.0: {:?}", ctx.version_file("1.0.0"));
    println!("Save 'quicksave': {:?}", ctx.save_dir("quicksave"));
    println!("Log file (now): {:?}", ctx.log_file_now());
    println!("Log file (custom): {:?}", ctx.log_file("20240315-120000"));
    println!();

    // Ensure directories exist
    match ctx.ensure_directories() {
        Ok(_) => println!("✓ All directories created successfully"),
        Err(e) => eprintln!("✗ Error creating directories: {}", e),
    }
    println!();

    // Example: Using PathContext in production vs development
    println!("=== Environment Detection ===");
    match ctx.environment() {
        RuntimeEnvironment::Development => {
            println!("Running in DEVELOPMENT mode");
            println!("→ Using project directory structure");
        }
        RuntimeEnvironment::Production => {
            println!("Running in PRODUCTION mode");
            println!("→ Using platform-specific app data directory");
        }
    }
}
