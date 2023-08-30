use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex as AsyncMutex;

use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::cli::*;
use crate::github::*;

use super::document::*;

pub struct Backend {
    pub client: Client,
    pub github: GithubWrapper,
    pub documents: Arc<AsyncMutex<HashMap<Url, Document>>>,
}

impl Backend {
    pub fn new(client: Client, cli: &Cli) -> Self {
        let github = GithubWrapper::new();
        if let Some(token) = &cli.github_token {
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
}
