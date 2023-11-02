mod cli;
mod clients;
mod lang;
mod server;
mod tools;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::Cli::new().run().await
}
