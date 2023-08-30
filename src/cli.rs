use clap::Parser;
use tracing::debug;

#[derive(Parser)]
pub struct Cli {
    #[arg(long, env)]
    pub github_token: Option<String>,
    #[arg(long)]
    pub stdio: bool,
}

impl Cli {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit_debug(&self) {
        debug!(
            "Parsed CLI arguments\n\tgithub_token: {}\n\tstdio: {}",
            self.github_token.is_some(),
            self.stdio
        )
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::parse()
    }
}
