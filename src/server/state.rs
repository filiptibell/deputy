use std::{collections::HashMap, ops::ControlFlow, time::Duration};

use tokio::time;
use tracing::debug;

use async_lsp::{router::Router, ClientSocket, Result};

use lsp_types::Url;

use super::events::*;
use crate::manifest::*;

pub struct Server {
    pub client: ClientSocket,
    pub manifests: HashMap<Url, Manifest>,
}

impl Server {
    pub fn new(client: ClientSocket) -> Self {
        let mut this = Self {
            client,
            manifests: HashMap::new(),
        };
        this.spawn_tick();
        this
    }

    pub fn into_router(self) -> Router<Self> {
        let mut router = Router::from_language_server(self);
        router.event(Self::on_tick);
        router
    }

    fn spawn_tick(&mut self) {
        let client = self.client.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if client.emit(TickEvent).is_err() {
                    break;
                }
            }
        });
    }

    fn on_tick(&mut self, _: TickEvent) -> ControlFlow<Result<()>> {
        debug!("tick");
        ControlFlow::Continue(())
    }

    pub fn update_document(&mut self, uri: Url, contents: String) -> ControlFlow<Result<()>> {
        debug!("Got text document {}", uri.path());
        if uri.path().contains("aftman.toml")
            || uri.path().contains("foreman.toml")
            || uri.path().contains("wally.toml")
        {
            debug!("Updating text document {}", uri.path());
            if let Ok(m) = Manifest::parse(contents) {
                self.manifests.insert(uri, m);
            }
        }
        ControlFlow::Continue(())
    }
}
