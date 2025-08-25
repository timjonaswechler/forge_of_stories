mod action;
mod app;
mod cli;
mod components;
mod config;
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
use config::ensure_data_and_config_dirs_exist;
use util::AppContext;

#[tokio::main]
pub async fn run() -> Result<()> {
    ensure_data_and_config_dirs_exist()?;
    crate::errors::init()?;
    crate::logging::init()?;
    let cx = AppContext::new();

    let mut app = App::new(4.0, 60.0, cx)?;
    app.run().await?;
    Ok(())
}
