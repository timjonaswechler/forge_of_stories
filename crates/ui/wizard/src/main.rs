mod action;
mod cli;
mod components;
mod core;
mod domain;
mod errors;
mod logging;
mod messages;
mod pages;
mod services;
mod theme;
mod tui;
mod ui;

use crate::cli::Cli;
use crate::core::app::WizardApp;
use crate::core::r#loop::AppLoop;

use clap::Parser;
use color_eyre::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Cli::parse();
    let base = app::init::<WizardApp>().expect("Inizialisation went wrong");
    let mut app = WizardApp::new(args, base)?;
    AppLoop::new(&mut app).run().await?;
    Ok(())
}
