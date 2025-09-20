mod action;
mod app;
mod cli;
mod components;
mod layers;
mod tui;

use crate::app::App as WizardApp;
use crate::cli::Cli;
use clap::Parser;
use color_eyre::Result;
use tracing_subscriber::EnvFilter;

// A zero-sized type implementing the platform `Application` trait used by `app::init`.
struct Wizard;

impl ::app::Application for Wizard {
    type Error = ::app::BoxError;

    // Stable application ID for config/data/logs directories.
    const APP_ID: &'static str = "wizard";

    fn init_platform() -> Result<(), Self::Error> {
        // Hook for platform-specific initialization if needed.
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install logging/tracing (env overrideable via RUST_LOG / RUST_TRACE)
    // Example: RUST_LOG=debug,wizard=debug,settings=debug
    let default_filter = "info,wizard=debug,settings=debug";
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter)),
        )
        .with_target(true)
        .compact()
        .init();

    let cli = Cli::parse();

    // Initialize platform base (paths, etc.)
    let base = crate::app::init::<Wizard>().expect("Initialization went wrong");

    // Build and run the Wizard TUI
    let mut app = WizardApp::new(base, cli)?;
    app.run().await?;
    Ok(())
}
