use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, server::LifecycleLayer, tracing::TracingLayer, MainLoop,
};

mod github;
mod manifest;
mod server;
mod stdio;
mod toml;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Set up logging - LSP uses stdout for communication,
    // meaning we must use stderr for all of our logging
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let tracing_target_enabled = matches!(
        tracing_filter.max_level_hint(),
        Some(LevelFilter::TRACE | LevelFilter::DEBUG)
    );
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_filter)
        .without_time()
        .with_target(tracing_target_enabled)
        .with_level(true)
        .with_ansi(false) // Editor output does not support ANSI ... yet?
        .with_writer(std::io::stderr)
        .init();

    // Create our language server
    let (server, _) = MainLoop::new_server(|client| {
        tower::ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client.clone()))
            .service(server::Server::new(client).into_router())
    });

    // Run it communicating over stdio, until the end of time
    let (stdin, stdout) = stdio::create();
    server
        .run_buffered(stdin, stdout)
        .await
        .expect("unexpected language server error");
}
