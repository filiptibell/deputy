use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex as AsyncMutex;

use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LspService, Server};

use crate::github::*;
use crate::util::*;

use super::document::*;
use super::notifications::*;

pub struct Backend {
    pub client: Client,
    pub github: GithubWrapper,
    pub documents: Arc<AsyncMutex<HashMap<Url, Document>>>,
}

impl Backend {
    fn new(client: Client, args: &Arguments) -> Self {
        let github = GithubWrapper::new();
        if let Some(token) = &args.github_token {
            github.set_auth_token(token);
        }

        let this = Self {
            client,
            github,
            documents: Arc::new(AsyncMutex::new(HashMap::new())),
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
                Server::new(read, write, socket).serve(service).await;
            }
            Transport::Stdio => {
                let (stdin, stdout) = create_stdio();
                Server::new(stdin, stdout, socket).serve(service).await;
            }
        }
    }
}
