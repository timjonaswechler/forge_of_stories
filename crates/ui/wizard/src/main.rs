mod action;
mod app;
mod cli;
mod components;
mod errors;
mod logging;
mod messages;
mod pages;
mod services;
mod settings;

mod tui;

use crate::app::App;
use crate::cli::Cli;
use clap::Parser;
use color_eyre::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();

    let mut app = App::new(args)?;
    app.run().await?;
    Ok(())
}
