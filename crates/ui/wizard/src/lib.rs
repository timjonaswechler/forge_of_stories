mod action;
mod app;
mod components;
mod config;
mod errors;
mod logging;
mod messages;
mod pages;
mod services;
mod state;
mod tui;

use crate::app::App;
use color_eyre::Result;

#[tokio::main]
pub async fn run() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let mut app = App::new()?;
    app.run().await?;
    Ok(())
}
