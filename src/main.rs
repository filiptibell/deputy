use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, server::LifecycleLayer, tracing::TracingLayer, MainLoop,
};

mod cli;
mod github;
mod manifest;
mod server;
mod toml;
mod util;

use cli::{Cli, Transport};
use server::Backend;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::new();

    // Set up logging / tracing
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("rustls=warn".parse().unwrap())
        .add_directive("tower=warn".parse().unwrap())
        .add_directive("octocrab=warn".parse().unwrap());
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_filter)
        .without_time()
        .with_target(false)
        .with_level(true)
        .with_ansi(false) // Editor output does not support ANSI ... yet?
        .with_writer(std::io::stderr) // Stdio transport takes up stdout, so emit output to stderr
        .init();

    // Create our language server
    let (server, _) = MainLoop::new_server(|client| {
        tower::ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client.clone()))
            .service(Backend::new(client, &cli).into_router())
    });

    // Run the server over the preferred transport protocol
    // FUTURE: Support pipe transport
    match cli.transport {
        Transport::Socket(port) => {
            let (read, write) = util::create_socket(port).await;
            server
                .run_buffered(read, write)
                .await
                .expect("unexpected language server error");
        }
        Transport::Stdio => {
            let (stdin, stdout) = util::create_stdio();
            server
                .run_buffered(stdin, stdout)
                .await
                .expect("unexpected language server error");
        }
    }
}
