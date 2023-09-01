use tokio::runtime::Builder;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

mod github;
mod server;
mod tools;
mod util;

use server::Server;
use util::Arguments;

fn main() {
    let args = Arguments::new();

    // Set up logging / tracing
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("rustls=warn".parse().unwrap())
        .add_directive("tower_lsp=warn".parse().unwrap())
        .add_directive("tower=info".parse().unwrap())
        .add_directive("octocrab=info".parse().unwrap());
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_filter)
        .without_time()
        .with_target(true)
        .with_level(true)
        .with_ansi(false) // Editor output does not support ANSI ... yet?
        .with_writer(std::io::stderr) // Stdio transport takes up stdout, so emit output to stderr
        .init();

    // Create and run our language server
    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime");
    rt.block_on(Server::serve(&args));
}
