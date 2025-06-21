mod cli;
mod server;
mod tracing;

use self::tracing::setup_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    cli::Cli::new().run().await
}
