use clap::Parser;
use tracing::debug;

use crate::util::Transport;

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

pub struct Arguments {
    pub transport: Transport,
    pub github_token: Option<String>,
}

impl Arguments {
    pub fn new() -> Self {
        let arguments = Inner::parse();

        let this = Self {
            transport: arguments.transport().unwrap_or_default(),
            github_token: arguments.github_token,
        };

        debug!(
            "Parsed arguments\n\ttransport: {}\n\tgithub_token: {}",
            this.transport,
            this.github_token.is_some(),
        );

        this
    }
}
