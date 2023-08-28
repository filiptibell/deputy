use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::time::uptime,
};

use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, server::LifecycleLayer, tracing::TracingLayer, MainLoop,
};

mod events;
mod manifest;
mod server;
mod state;
mod stdio;

#[tokio::main]
async fn main() {
    // Set up logging - LSP uses stdout for communication,
    // meaning we must use stderr for all of our logging
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_filter)
        .with_target(true)
        .with_timer(uptime())
        .with_level(true)
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
            .service(state::ServerState::new(client).into_router())
    });

    // Run it communicating over stdio, until the end of time
    let (stdin, stdout) = stdio::create();
    server
        .run_buffered(stdin, stdout)
        .await
        .expect("unexpected language server error");
}
