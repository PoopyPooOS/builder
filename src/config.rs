#![allow(dead_code)] // My eyes hurt, TODO: Remove this once all the fields are used

use log::error;
use serde::Deserialize;
use std::{path::PathBuf, process};
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub builder: BuilderConfig,
    pub runner: RunnerConfig,
}

#[derive(Debug, Deserialize)]
pub struct BuilderConfig {
    pub build_target: String,
    pub components_dir: PathBuf,
    pub rootfs_dir: PathBuf,
    pub dist_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct RunnerConfig {
    pub kernel_args: String,
    pub qemu_bin: Option<PathBuf>,
    pub qemu_args: Option<Vec<String>>,
}

impl Config {
    pub async fn read(path: impl Into<PathBuf>) -> Self {
        let path: PathBuf = path.into();
        let raw = match fs::read_to_string(path).await {
            Ok(raw) => raw,
            Err(err) => {
                error!("Failed to read config: {}", err);
                process::exit(1)
            }
        };

        match toml::from_str(&raw) {
            Ok(config) => config,
            Err(err) => {
                error!("Failed to parse config: {}", err);
                process::exit(1)
            }
        }
    }
}
