mod action;
mod cli;
mod components;
mod errors;
mod logging;
mod messages;
mod pages;
mod services;
mod settings;
mod tui;
mod wizard;

use crate::cli::Cli;
use crate::wizard::App;

use clap::Parser;
use color_eyre::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Cli::parse();
    let base = app::init::<App>().expect("Inizialisation went wrong");
    let mut app = App::new(args, base)?;
    app.run().await?;
    Ok(())
}
