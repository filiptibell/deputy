use std::process::ExitCode;

mod cli;
mod server;
mod tracing;

use self::tracing::setup_tracing;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<ExitCode> {
    let cli = cli::Cli::new();
    setup_tracing(cli.quiet_tracing());
    cli.run().await
}
