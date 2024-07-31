use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    pub fn command(&self) -> Command {
        self.command.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Run {
        #[arg(long)]
        iso: Option<bool>,
    },
    R {
        #[arg(long)]
        iso: Option<bool>,
    },
    Build {
        #[arg(long)]
        no_run: bool,
        #[arg(long)]
        iso: bool,
    },
    B {
        #[arg(long)]
        no_run: bool,
        #[arg(long)]
        iso: bool,
    },
    Add {
        name: String,
        out: PathBuf,
    },
    A {
        name: String,
        out: PathBuf,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Build { no_run: false, iso: false }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub build_target: String,
    pub components_dir: PathBuf,
    pub rootfs_dir: PathBuf,
    pub dist_dir: PathBuf,
    pub qemu_args: Vec<String>,
    pub kernel_args: String,
}

#[derive(Debug, Default, Deserialize)]
pub enum BuildType {
    Debug,
    #[default]
    Release,
}

#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub path: PathBuf,
    pub component_type: ComponentType,

    pub config: Option<toml::Table>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComponentType {
    Binary,
    Module,
    Other,
}
