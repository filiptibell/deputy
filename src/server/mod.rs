use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;

use tower_lsp::{Client, LspService, Server as LspServer};

use crate::clients::*;
use crate::tools::*;

mod document;
mod initialize;
mod language_server;
mod requests;
mod transport;

pub use document::*;
pub use transport::*;

#[derive(Debug, Clone)]
pub struct ServerArguments {
    pub transport: Transport,
    pub github_token: Option<String>,
}

pub struct ServerInner {
    client: Client,
    clients: Clients,
    documents: Documents,
    tools: Tools,
}

pub struct Server {
    args: ServerArguments,
    inner: Option<ServerInner>,
}

impl Deref for Server {
    type Target = ServerInner;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("missing inner struct")
    }
}

impl DerefMut for Server {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect("missing inner struct")
    }
}

impl Server {
    pub fn new(args: ServerArguments) -> Self {
        Self { args, inner: None }
    }

    fn attach_to(&mut self, client: Client) {
        let clients = Clients::new();
        let documents = Arc::new(DashMap::new());

        if let Some(token) = &self.args.github_token {
            clients.github.set_auth_token(token);
        }

        self.inner.replace(ServerInner {
            client: client.clone(),
            clients: clients.clone(),
            documents: Arc::clone(&documents),
            tools: Tools::new(client, clients, documents),
        });

        self.watch_rate_limit();
    }

    pub async fn serve(mut self) -> Result<()> {
        // FUTURE: Add custom notifications here by using
        // LspService::build and calling custom_method
        let (service, socket) = LspService::new(|client| {
            self.attach_to(client);
            self
        });

        match service.inner().args.transport {
            Transport::Socket(port) => {
                let (read, write) = Transport::create_socket(port).await;
                LspServer::new(read, write, socket).serve(service).await;
            }
            Transport::Stdio => {
                let (stdin, stdout) = Transport::create_stdio();
                LspServer::new(stdin, stdout, socket).serve(service).await;
            }
        }

        Ok(())
    }
}
