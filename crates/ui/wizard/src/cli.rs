// src/cli.rs
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "wizard", version, about = "Forge of Stories admin tool")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Run interactive TUI
    Run {
        #[command(subcommand)]
        mode: RunMode,
    },
    /// Health probe (scripts/monitoring)
    Health,
    // /// Install components
    // Install {
    //     #[arg(value_enum)]
    //     what: InstallWhat,
    //     version: String,
    // },
    // /// List installed items / versions / mods
    // List {
    //     #[arg(value_enum)]
    //     what: ListWhat,
    //     #[arg(long)]
    //     json: bool,
    // },
}

#[derive(Subcommand)]
pub enum RunMode {
    Setup,
    Dashboard,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum LifecycleAction {
    Start,
    Stop,
    Restart,
}

#[derive(Clone, ValueEnum)]
pub enum InstallWhat {
    Aether,
    Dlc,
    Mod,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum ListWhat {
    Versions,
    Dlcs,
    Mods,
}
