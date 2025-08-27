mod action;
mod app;
mod cli;
mod components;
mod errors;
mod logging;
mod messages;
mod pages;
mod services;
mod style;
mod tui;
mod utils;

use crate::app::App;
use color_eyre::Result;

#[tokio::main]
pub async fn run() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let mut app = App::new(4.0, 60.0)?;
    app.run().await?;
    Ok(())
}
