use std::io::{IsTerminal, stderr};

use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[cfg(debug_assertions)]
const IS_DEBUG: bool = true;
#[cfg(not(debug_assertions))]
const IS_DEBUG: bool = false;

pub fn setup_tracing(quiet: bool) {
    let default_directive = if quiet {
        LevelFilter::OFF.into()
    } else if IS_DEBUG {
        LevelFilter::DEBUG.into()
    } else {
        LevelFilter::INFO.into()
    };
    let mut tracing_filter = EnvFilter::builder()
        .with_default_directive(default_directive)
        .from_env_lossy();

    if !quiet {
        tracing_filter = tracing_filter
            .add_directive("async_language_server=warn".parse().unwrap())
            .add_directive("globset=warn".parse().unwrap())
            .add_directive("ignore=warn".parse().unwrap())
            .add_directive("tower_lsp=warn".parse().unwrap())
            .add_directive("tower=info".parse().unwrap());
    }

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
