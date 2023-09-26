use tracing_subscriber::filter::{EnvFilter, LevelFilter};

mod clients;
mod server;
mod tools;
mod util;

use server::Server;
use util::Arguments;

#[cfg(debug)]
const IS_DEBUG: bool = true;
#[cfg(not(debug))]
const IS_DEBUG: bool = false;

#[tokio::main]
async fn main() {
    // Set up logging / tracing
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("rustls=warn".parse().unwrap())
        .add_directive("tower_lsp=warn".parse().unwrap())
        .add_directive("tower=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap());
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_filter)
        .without_time()
        .with_target(IS_DEBUG)
        .with_level(true)
        .with_ansi(false) // Editor output does not support ANSI ... yet?
        .with_writer(std::io::stderr) // Stdio transport takes up stdout, so emit output to stderr
        .init();

    // Parse startup arguments
    let args = Arguments::new();

    // Create and run our language server
    Server::serve(&args).await;
}
