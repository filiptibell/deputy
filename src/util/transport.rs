use std::net::SocketAddr;

use tokio::{
    io::{stdin, stdout, AsyncRead, AsyncWrite},
    net::TcpSocket,
};

/**
    Creates a socket listener.
*/
pub async fn create_socket(port: u16) -> (impl AsyncRead, impl AsyncWrite) {
    let addr = SocketAddr::try_from(([127, 0, 0, 1], port)).unwrap();
    let socket = TcpSocket::new_v4().expect("Failed to crate tcp socket");

    let stream = socket
        .connect(addr)
        .await
        .expect("Failed to connect to socket");

    stream.into_split()
}

/**
    Get handles to standard input and output streams.
*/
pub fn create_stdio() -> (impl AsyncRead, impl AsyncWrite) {
    (stdin(), stdout())
}
