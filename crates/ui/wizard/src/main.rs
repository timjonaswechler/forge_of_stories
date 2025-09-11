mod action;
mod app;
mod cli;
mod components;
mod layers;
mod pages;
mod tui;

use clap::Parser;
use color_eyre::Result;

// Use the external platform crate via absolute path to avoid confusion with the local `app` module.

// Bring our application type into scope with a clear alias.
use crate::app::App as WizardApp;
use crate::cli::Cli;

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
    let cli = Cli::parse();

    // Use the re-exported `init` function from our local module, but provide a type
    // that implements the external platform `Application` trait.
    let base = crate::app::init::<Wizard>().expect("Initialization went wrong");

    // Fix argument order to match `App::new(base, cli)`.
    let mut app = WizardApp::new(base, cli)?;
    app.run().await?;
    Ok(())
}
