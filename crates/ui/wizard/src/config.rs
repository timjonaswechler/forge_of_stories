#![allow(dead_code)] // Remove this once you start using the code

use color_eyre::Result;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use paths::{config_dir, data_dir};
use serde::Deserialize;
use std::fs;
use std::{env, path::PathBuf};
use tracing::error;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub config_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        let data_dir = data_dir();
        let config_dir = config_dir();
        let mut builder = config::Config::builder()
            .set_default("data_dir", data_dir.to_str().unwrap())?
            .set_default("config_dir", config_dir.to_str().unwrap())?;

        let config_files = [
            ("config.json5", config::FileFormat::Json5),
            ("config.toml", config::FileFormat::Toml),
        ];
        let mut found_config = false;
        for (file, format) in &config_files {
            let source = config::File::from(config_dir.join(file))
                .format(*format)
                .required(false);
            builder = builder.add_source(source);
            if config_dir.join(file).exists() {
                found_config = true
            }
        }
        if !found_config {
            error!("No configuration file found. Application may not behave as expected");
        }

        let cfg: Self = builder.build()?.try_deserialize()?;

        Ok(cfg)
    }
}
