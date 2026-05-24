use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod analyze;
mod serve;

use self::analyze::AnalyzeCommand;
use self::serve::ServeCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Analyze(AnalyzeCommand),
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

    pub fn quiet_tracing(&self) -> bool {
        matches!(self.subcommand, CliSubcommand::Analyze(_))
    }

    pub async fn run(self) -> Result<ExitCode> {
        match self.subcommand {
            CliSubcommand::Analyze(cmd) => cmd.run().await,
            CliSubcommand::Serve(cmd) => {
                cmd.run().await?;
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}
