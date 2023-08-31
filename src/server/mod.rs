use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex as AsyncMutex;

use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::{Client, LspService, Server as LspServer};

use crate::github::*;
use crate::tools::*;
use crate::util::*;

mod document;
mod initialize;
mod language_server;
mod notifications;

use notifications::*;

pub use document::*;

pub struct Server {
    pub client: Client,
    pub github: GithubWrapper,
    pub documents: Documents,
    pub tools: Tools,
}

impl Server {
    fn new(client: Client, args: &Arguments) -> Self {
        let github = GithubWrapper::new();
        if let Some(token) = &args.github_token {
            github.set_auth_token(token);
        }

        let documents = Arc::new(AsyncMutex::new(HashMap::new()));

        let this = Self {
            client: client.clone(),
            github: github.clone(),
            documents: Arc::clone(&documents),
            tools: Tools::new(client, github, documents),
        };

        this.watch_rate_limit();
        this
    }

    pub async fn serve(args: &Arguments) {
        let (service, socket) = LspService::build(|client| Self::new(client, args))
            .custom_method(RateLimitNotification::METHOD, Self::on_notified_rate_limit)
            .finish();

        match args.transport {
            Transport::Socket(port) => {
                let (read, write) = create_socket(port).await;
                LspServer::new(read, write, socket).serve(service).await;
            }
            Transport::Stdio => {
                let (stdin, stdout) = create_stdio();
                LspServer::new(stdin, stdout, socket).serve(service).await;
            }
        }
    }
}
