mod runtime;
mod settings;
mod wizard;

use crate::settings::ServerConfig;
use color_eyre::Result;
use std::fs;

fn main() -> Result<()> {
    // Check if we need to run setup wizard
    let (mut config, should_run_setup) = should_run_setup();

    if should_run_setup {
        // If we need to run the setup wizard, we do it here
        // match crate::wizard::run() {
        //     Ok(c) => config = c,
        //     Err(e) => {
        //         eprintln!("Error running setup wizard: {}", e);
        //         exit(1);
        //     }
        // }
        //
        println!("Starting setup wizard ...");
        crate::wizard::run();
    }

    // Continue with normal server startup
    println!("Starting Server ...");
    start_server(config)?;

    Ok(())
}

fn should_run_setup() -> (ServerConfig, bool) {
    let config_path = "assets/server.toml";
    match fs::read_to_string(config_path) {
        Ok(content) => {
            // Try to deserialize the config from TOML
            match toml::from_str::<ServerConfig>(&content) {
                Ok(config) => (config, false),
                Err(_) => {
                    // If parsing fails, run setup wizard with default config
                    (ServerConfig::default(), true)
                }
            }
        }
        Err(_) => {
            // If the file does not exist, we need to run the setup wizard
            (ServerConfig::default(), true)
        }
    }
}

fn start_server(config: ServerConfig) -> Result<()> {
    println!("Starting server with args: {:?}", config);
    // TODO: Implement actual server startup
    Ok(())
}
