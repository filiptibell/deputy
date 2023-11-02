use std::{fmt, net::SocketAddr};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

/**
    Transport implementation for sockets and stdio.
*/
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub enum Transport {
    Socket(u16),
    #[default]
    Stdio,
}

impl Transport {
    /**
        Creates a socket listener.
    */
    pub async fn create_socket(port: u16) -> (impl AsyncRead, impl AsyncWrite) {
        let addr = SocketAddr::try_from(([127, 0, 0, 1], port)).unwrap();

        let stream = TcpStream::connect(addr)
            .await
            .expect("Failed to connect to socket");

        stream.into_split()
    }

    /**
        Get handles to standard input and output streams.
    */
    pub fn create_stdio() -> (impl AsyncRead, impl AsyncWrite) {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        (stdin, stdout)
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdio => write!(f, "Stdio"),
            Self::Socket(p) => write!(f, "Socket({p})"),
        }
    }
}
