use clap::Parser;
use types::{Cli, Command, Config};

mod builder;
mod parser;
mod runner;
mod types;

fn main() {
    let command = Cli::parse().command();

    let config: Config = parser::parse("builder.toml");

    match command {
        Command::Run { iso } | Command::R { iso } => {
            runner::run(&config, iso.unwrap_or_default());
        }
        Command::Build { no_run, iso } | Command::B { no_run, iso } => {
            builder::build(&config, iso).expect("Build failed");

            if !no_run {
                runner::run(&config, iso);
            }
        }
        _ => (),
    }
}
