use clap::Parser;
use std::{fs, path::PathBuf};
use types::{Cli, Command, Config};
use which::which;

mod builder;
mod parser;
mod runner;
mod types;
mod utils;

fn main() {
    let mut missing: Vec<&str> = vec![];

    for dep in &["cargo", "qemu-system-x86_64", "sh", "find", "cpio", "grub-mkrescue", "gh"] {
        which(dep).is_err().then(|| missing.push(dep));
    }

    assert!(missing.is_empty(), "Missing dependencies: {}.", missing.join(", "));

    let builder_cache = PathBuf::from(".builder");

    if !builder_cache.exists() {
        fs::create_dir_all(&builder_cache).expect("Failed to create build tmp dir");
    }

    let command = Cli::parse().command();
    let config: Config = parser::parse("builder.toml");

    match command {
        Command::Run { iso } | Command::R { iso } => {
            runner::run(&config, iso);
        }
        Command::Build { no_run, iso } | Command::B { no_run, iso } => {
            builder::build(&config, iso).expect("Build failed");

            if !no_run {
                runner::run(&config, iso);
            }
        }
    }
}
