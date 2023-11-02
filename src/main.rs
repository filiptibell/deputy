mod cli;
mod clients;
mod lang;
mod server;
mod tools;
mod util;

#[tokio::main]
async fn main() {
    cli::Cli::new().run().await;
}
