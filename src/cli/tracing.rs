use std::io::{stderr, IsTerminal};

use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[cfg(debug_assertions)]
const IS_DEBUG: bool = true;
#[cfg(not(debug_assertions))]
const IS_DEBUG: bool = false;

pub fn setup_tracing() {
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("rustls=warn".parse().unwrap())
        .add_directive("tower_lsp=warn".parse().unwrap())
        .add_directive("tower=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap());

    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_filter)
        .without_time()
        .with_target(IS_DEBUG)
        .with_level(true)
        .with_ansi(stderr().is_terminal())
        .with_writer(stderr) // Stdio transport takes up stdout, so emit output to stderr
        .init();
}
