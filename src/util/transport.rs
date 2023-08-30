use std::net::SocketAddr;

use futures::{AsyncRead, AsyncWrite};

use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpSocket,
};

use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

/**
    Creates a socket listener.
*/
pub async fn create_socket(port: u16) -> (Compat<OwnedReadHalf>, Compat<OwnedWriteHalf>) {
    let addr = SocketAddr::try_from(([127, 0, 0, 1], port)).unwrap();
    let socket = TcpSocket::new_v4().expect("Failed to crate tcp socket");

    let stream = socket
        .connect(addr)
        .await
        .expect("Failed to connect to socket");

    let (read, write) = stream.into_split();
    (
        TokioAsyncReadCompatExt::compat(read),
        TokioAsyncWriteCompatExt::compat_write(write),
    )
}

/**
    Get handles to standard input and output streams.

    Prefers truly asynchronous piped stdin/stdout without blocking
    tasks, falls back to spawn blocking read/write otherwise.
*/
pub fn create_stdio() -> (impl AsyncRead, impl AsyncWrite) {
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
