#![feature(let_chains)]

use clap::Parser;
use cli::{Cli, Command};
use config::Config;
use log::{error, trace, LevelFilter};
use std::process;

mod cli;
mod commands;
mod config;
mod parser;
mod utils;

#[tokio::main]
async fn main() {
    logger_init();

    trace!("Checking for missing dependencies");
    check_dependencies();

    let command = Cli::parse().command();
    let config = Config::read("builder.toml").await;

    match command {
        Command::Run { iso } => {
            commands::run(&config, iso).await;
        }
        Command::Build { iso, no_run } => {
            match commands::build(&config, iso).await {
                Ok(()) => {}
                Err(err) => {
                    error!("Build failed: {err}");
                    process::exit(1);
                }
            };

            if !no_run {
                commands::run(&config, iso).await;
            }
        }
    }
}

fn check_dependencies() {
    let mut missing = Vec::<&str>::new();

    for dep in &["sh", "cargo", "qemu-system-x86_64", "find", "cpio", "grub-mkrescue"] {
        which::which(dep).is_err().then(|| missing.push(dep));
    }

    if !missing.is_empty() {
        error!(
            "Missing dependencies: {}.\nYou can install them with your distro's package manager or by using the nix dev shell in the project root.",
            missing.join(", ")
        );

        process::exit(0);
    }
}

fn logger_init() {
    colog::default_builder()
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();
}
