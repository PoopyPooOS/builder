use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub fn command(&mut self) -> Command {
        self.command.take().unwrap_or_default()
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run {
        #[arg(long)]
        iso: bool,
    },
    R {
        #[arg(long)]
        iso: bool,
    },
    Build {
        #[arg(short, long)]
        no_run: bool,
        #[arg(short, long)]
        iso: bool,
    },
    B {
        #[arg(short, long)]
        no_run: bool,
        #[arg(short, long)]
        iso: bool,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Build { no_run: false, iso: false }
    }
}

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

#[derive(Debug, Default, Deserialize)]
pub enum BuildType {
    Debug,
    #[default]
    Release,
}

impl Display for BuildType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildType::Debug => write!(f, "debug"),
            BuildType::Release => write!(f, "release"),
        }
    }
}

#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub path: PathBuf,
    #[allow(clippy::struct_field_names)]
    pub component_type: ComponentType,

    pub config: Option<toml::Table>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentType {
    Binary,
    Module,
    Other,
}

#[derive(Debug, Deserialize)]
pub struct BinaryComponentConfig {
    pub out: PathBuf,
    #[serde(default)]
    pub build_type: BuildType,
}
