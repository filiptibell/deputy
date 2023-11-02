use ::tracing::debug;
use clap::Parser;

use crate::server::{Server, ServerArguments};
use crate::util::Transport;

mod tracing;

#[derive(Parser)]
struct Inner {
    #[arg(long, alias = "port")]
    pub socket: Option<u16>,
    #[arg(long)]
    pub stdio: bool,
    #[arg(long, env)]
    pub github_token: Option<String>,
}

impl Inner {
    pub fn transport(&self) -> Option<Transport> {
        if let Some(port) = self.socket {
            Some(Transport::Socket(port))
        } else if self.stdio {
            Some(Transport::Stdio)
        } else {
            None
        }
    }
}

pub struct Cli {
    pub transport: Transport,
    pub github_token: Option<String>,
}

impl Cli {
    pub fn new() -> Self {
        let arguments = Inner::parse();

        let this = Self {
            transport: arguments.transport().unwrap_or_default(),
            github_token: arguments.github_token,
        };

        debug!(
            "Parsed arguments\n\ttransport: {}\n\tgithub_token: {}",
            this.transport,
            if this.github_token.is_some() {
                "Some(_)"
            } else {
                "None"
            },
        );

        this
    }

    pub async fn run(self) {
        // Set up tracing
        tracing::setup_tracing();

        // Create and run our language server
        Server::serve(&ServerArguments {
            transport: self.transport,
            github_token: self.github_token,
        })
        .await;
    }
}
