use std::net::{TcpStream, ToSocketAddrs};

use http_types::{Request, Response};
use smol::Async;
use thiserror::Error;
use url::Url;

pub type ClientResult<T, E = ClientError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ParseUrl(#[from] url::ParseError),
    #[error("failed to parse host")]
    ParseHost,
    #[error("failed to parse port")]
    ParsePort,
    #[error("failed to resolve address")]
    ResolveAddress,
    #[error("unsupported scheme '{0}'")]
    UnsupportedScheme(String),
    #[error("stream error - {0}")]
    Stream(String),
}

fn parse_url(url: &Url) -> ClientResult<(String, u16)> {
    let host = url.host().ok_or(ClientError::ParseHost)?.to_string();
    let port = url.port_or_known_default().ok_or(ClientError::ParsePort)?;
    Ok((host, port))
}

async fn connect(host: String, port: u16) -> ClientResult<Async<TcpStream>> {
    let socket_addr = smol::unblock(move || (host, port).to_socket_addrs())
        .await?
        .next()
        .ok_or(ClientError::ResolveAddress)?;

    let stream = Async::<TcpStream>::connect(socket_addr).await?;

    Ok(stream)
}

async fn send(stream: Async<TcpStream>, host: String, request: Request) -> ClientResult<Response> {
    match request.url().scheme() {
        "http" => async_h1::connect(stream, request)
            .await
            .map_err(|e| ClientError::Stream(e.to_string())),
        "https" => {
            let stream = async_native_tls::connect(&host, stream)
                .await
                .map_err(|e| ClientError::Stream(e.to_string()))?;
            async_h1::connect(stream, request)
                .await
                .map_err(|e| ClientError::Stream(e.to_string()))
        }
        scheme => Err(ClientError::UnsupportedScheme(scheme.to_string())),
    }
}

pub async fn fetch(req: Request) -> ClientResult<Response> {
    let (host, port) = parse_url(req.url())?;
    let stream = connect(host.clone(), port).await?;
    send(stream, host, req).await
}
