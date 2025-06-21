use anyhow::Result;
use clap::{Parser, Subcommand};

mod serve;

use self::serve::ServeCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Serve(ServeCommand),
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: CliSubcommand,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn run(self) -> Result<()> {
        match self.subcommand {
            CliSubcommand::Serve(cmd) => cmd.run().await,
        }
    }
}
