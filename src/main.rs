use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use tower_lsp::{lsp_types::notification::Notification, LspService, Server};

mod cli;
mod github;
mod manifest;
mod server;
mod toml;
mod util;

use cli::{Cli, Transport};
use server::{notifications::RateLimitNotification, Backend};

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
    let (service, socket) = LspService::build(|client| Backend::new(client, &cli))
        .custom_method(
            RateLimitNotification::METHOD,
            Backend::on_notified_rate_limit,
        )
        .finish();

    // Run the server over the preferred transport protocol
    // FUTURE: Support pipe transport
    match cli.transport {
        Transport::Socket(port) => {
            let (read, write) = util::create_socket(port).await;
            Server::new(read, write, socket).serve(service).await;
        }
        Transport::Stdio => {
            let (stdin, stdout) = util::create_stdio();
            Server::new(stdin, stdout, socket).serve(service).await;
        }
    }
}
