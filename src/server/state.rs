use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex as AsyncMutex;

use async_lsp::{router::Router, ClientSocket};

use lsp_types::Url;

use crate::cli::*;
use crate::github::*;

use super::document::*;
use super::notifications::*;

pub struct Server {
    pub client: ClientSocket,
    pub github: GithubWrapper,
    pub documents: Arc<AsyncMutex<HashMap<Url, Document>>>,
    pub workspace_folders: Vec<(String, Url)>,
}

impl Server {
    pub fn new(client: ClientSocket, cli: &Cli) -> Self {
        let mut github = GithubWrapper::new();
        if let Some(token) = &cli.github_token {
            github.set_auth_token(token);
        }

        let mut this = Self {
            client,
            github,
            documents: Arc::new(AsyncMutex::new(HashMap::new())),
            workspace_folders: Vec::new(),
        };

        this.spawn_rate_limit();
        this.spawn_tick();

        this
    }

    pub fn into_router(self) -> Router<Self> {
        let mut router = Router::from_language_server(self);
        router.notification::<RateLimitNotification>(Self::on_notified_rate_limit);
        router.event(Self::on_event_rate_limit);
        router.event(Self::on_event_tick);
        router
    }
}
