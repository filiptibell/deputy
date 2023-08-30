use futures::{AsyncRead, AsyncWrite};

/**
    Get handles to standard input and output streams.

    Prefers truly asynchronous piped stdin/stdout without blocking
    tasks, falls back to spawn blocking read/write otherwise.
*/
pub fn create() -> (impl AsyncRead, impl AsyncWrite) {
    #[cfg(unix)]
    {
        use async_lsp::stdio::{PipeStdin, PipeStdout};

        (
            PipeStdin::lock_tokio().expect("Failed to create stdio"),
            PipeStdout::lock_tokio().expect("Failed to create stdio"),
        )
    }

    #[cfg(not(unix))]
    {
        use tokio::io::{stdin, stdout};
        use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

        (
            TokioAsyncReadCompatExt::compat(stdin()),
            TokioAsyncWriteCompatExt::compat_write(stdout()),
        )
    }
}
