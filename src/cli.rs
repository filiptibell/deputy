#![allow(dead_code)]

use std::fmt;

use clap::Parser;
use tracing::debug;

#[derive(Parser)]
struct Arguments {
    #[arg(long, alias = "port")]
    pub socket: Option<u16>,
    #[arg(long)]
    pub stdio: bool,
    #[arg(long, env)]
    pub github_token: Option<String>,
}

impl Arguments {
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
        let arguments = Arguments::parse();

        let this = Self {
            transport: arguments.transport().unwrap_or_default(),
            github_token: arguments.github_token,
        };

        debug!(
            "Parsed CLI arguments\n\ttransport: {}\n\tgithub_token: {}",
            this.transport,
            this.github_token.is_some(),
        );

        this
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub enum Transport {
    Socket(u16),
    #[default]
    Stdio,
}

impl Transport {
    pub fn is_socket(&self) -> bool {
        matches!(self, Self::Socket(_))
    }

    pub fn is_stdio(&self) -> bool {
        matches!(self, Self::Stdio)
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdio => write!(f, "Stdio"),
            Self::Socket(p) => write!(f, "Socket({p})"),
        }
    }
}
