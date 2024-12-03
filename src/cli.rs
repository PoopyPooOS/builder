use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
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
    #[command(alias = "r")]
    Run {
        #[arg(long)]
        iso: bool,
    },
    #[command(alias = "b")]
    Build {
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
