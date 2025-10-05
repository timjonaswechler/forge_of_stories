//! Bevy application example demonstrating Bevy integration with AppBuilder.
//!
//! This example shows how to create a Bevy-based application using the
//! AppBuilder pattern with the `build_with_bevy()` method.

use app::{AppBuilder, Application};

#[cfg(feature = "bevy")]
use bevy::prelude::*;

// Define your application
struct MyBevyApp;

impl Application for MyBevyApp {
    const APP_ID: &'static str = "my_bevy_app";
    // Uses defaults: STUDIO = "chicken105", PROJECT_ID = "forge_of_stories"
}

#[cfg(feature = "bevy")]
fn main() -> Result<(), app::BoxError> {
    println!("=== Bevy App Example ===\n");

    // Initialize the application with Bevy
    let mut bevy_app = AppBuilder::<MyBevyApp>::new("1.0.0")?.build_with_bevy(|mut app, ctx| {
        println!("🎮 Configuring Bevy app...");
        println!("   App ID: {}", ctx.app_id());
        println!("   Version: {}", ctx.version());
        println!("   Assets Dir: {:?}", ctx.path_context().assets_dir());
        println!();

        // Configure the Bevy app with minimal plugins
        app.add_plugins(MinimalPlugins);
        app.add_systems(Startup, setup_system);
        app.add_systems(Update, demo_system);

        app
    });

    println!("✅ Bevy app initialized successfully!\n");
    println!("🚀 Running Bevy app (press Ctrl+C to stop)...\n");

    // Run the Bevy app
    bevy_app.run();

    Ok(())
}

#[cfg(feature = "bevy")]
fn setup_system() {
    println!("🔧 Startup system running!");
}

#[cfg(feature = "bevy")]
fn demo_system(mut counter: Local<u32>) {
    *counter += 1;

    if *counter == 1 || *counter % 60 == 0 {
        println!("⚙️  Update system tick: {}", *counter);
    }

    // Exit after a few seconds
    if *counter >= 180 {
        println!("\n✨ Demo completed successfully!");
        std::process::exit(0);
    }
}

#[cfg(not(feature = "bevy"))]
fn main() {
    eprintln!("❌ This example requires the 'bevy' feature to be enabled.");
    eprintln!("   Run with: cargo run -p app --example bevy_app --features bevy");
    std::process::exit(1);
}
