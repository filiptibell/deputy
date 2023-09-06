use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex as AsyncMutex;

use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::{Client, LspService, Server as LspServer};

use crate::crates::*;
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
    pub documents: Documents,
    pub github: GithubWrapper,
    pub crates: CratesWrapper,
    pub tools: Tools,
}

impl Server {
    fn new(client: Client, args: &Arguments) -> Self {
        let mut rheaders = reqwest::header::HeaderMap::new();
        rheaders.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static(concat!(
                env!("CARGO_PKG_NAME"),
                "@",
                env!("CARGO_PKG_VERSION")
            )),
        );
        let rclient = reqwest::Client::builder()
            .default_headers(rheaders)
            .build()
            .expect("Failed to create reqwest client");

        let github = GithubWrapper::new(rclient.clone());
        let crates = CratesWrapper::new(rclient.clone());

        let documents = Arc::new(AsyncMutex::new(HashMap::new()));

        if let Some(token) = &args.github_token {
            github.set_auth_token(token);
        }

        let this = Self {
            client: client.clone(),
            documents: Arc::clone(&documents),
            github: github.clone(),
            crates: crates.clone(),
            tools: Tools::new(client, documents, github, crates),
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
