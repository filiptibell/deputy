mod cli;
mod clients;
mod parser;
mod server;
mod tools;
mod tracing;
mod util;

use self::tracing::setup_tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    cli::Cli::new().run().await
}
