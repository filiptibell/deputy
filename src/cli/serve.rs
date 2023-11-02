use anyhow::Result;
use clap::Parser;
use tracing::debug;

use crate::server::{Server, ServerArguments, Transport};

#[derive(Debug, Clone, Parser)]
pub struct ServeCommand {
    #[arg(long, alias = "port")]
    pub socket: Option<u16>,
    #[arg(long)]
    pub stdio: bool,
    #[arg(long, env)]
    pub github_token: Option<String>,
}

impl ServeCommand {
    pub async fn run(self) -> Result<()> {
        let transport = if let Some(port) = self.socket {
            Some(Transport::Socket(port))
        } else if self.stdio {
            Some(Transport::Stdio)
        } else {
            None
        };

        let args = ServerArguments {
            transport: transport.unwrap_or_default(),
            github_token: self.github_token,
        };

        debug!(
            "Parsed arguments\n\ttransport: {}\n\tgithub_token: {}",
            args.transport,
            if args.github_token.is_some() {
                "Some(_)"
            } else {
                "None"
            },
        );

        Server::new(args).serve().await
    }
}
