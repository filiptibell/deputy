use std::io::{stderr, IsTerminal};

use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[cfg(debug_assertions)]
const IS_DEBUG: bool = true;
#[cfg(not(debug_assertions))]
const IS_DEBUG: bool = false;

pub fn setup_tracing() {
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(if IS_DEBUG {
            LevelFilter::DEBUG.into()
        } else {
            LevelFilter::INFO.into()
        })
        .from_env_lossy()
        .add_directive("tower_lsp=warn".parse().unwrap())
        .add_directive("tower=info".parse().unwrap());

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
