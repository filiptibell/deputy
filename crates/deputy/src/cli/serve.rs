use anyhow::{Context, Result};
use clap::Parser;
use tracing::debug;

use async_language_server::server::{Transport, serve};

use crate::server::DeputyLanguageServer;

#[derive(Debug, Clone, Parser)]
pub struct ServeCommand {
    #[arg(long, alias = "port")]
    pub socket: Option<u16>,
    #[arg(long)]
    pub stdio: bool,
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

        let transport = transport.unwrap_or_default();
        let server = DeputyLanguageServer::new();

        debug!("Parsed arguments\n\ttransport: {transport}");

        serve(transport, server)
            .await
            .context("encountered fatal error - language server shutting down")
    }
}
