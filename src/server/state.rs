use std::{collections::HashMap, ops::ControlFlow, path::PathBuf, sync::Arc, time::Duration};

use futures::future::join_all;
use tokio::{sync::Mutex as AsyncMutex, time};
use tracing::{debug, trace, warn};

use async_lsp::{router::Router, ClientSocket, Result};

use lsp_types::Url;

use super::events::*;
use crate::manifest::*;

const KNOWN_FILE_NAMES: &[&str] = &["aftman.toml", "foreman.toml", "wally.toml"];

pub struct Server {
    pub client: ClientSocket,
    pub manifests: Arc<AsyncMutex<HashMap<Url, Manifest>>>,
}

impl Server {
    pub fn new(client: ClientSocket) -> Self {
        let mut this = Self {
            client,
            manifests: Arc::new(AsyncMutex::new(HashMap::new())),
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

    pub fn update_document(&self, uri: Url, contents: String) -> ControlFlow<Result<()>> {
        let file_path = match uri.to_file_path() {
            Err(_) => return ControlFlow::Continue(()),
            Ok(path) => path,
        };

        debug!("Updating document: {}", file_path.display());

        let manifests = Arc::clone(&self.manifests);
        tokio::task::spawn(async move {
            let mut manifests = manifests.lock().await;
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                if KNOWN_FILE_NAMES.contains(&file_name) {
                    parse_and_insert(&mut manifests, file_path, Ok(contents));
                }
            }
        });

        ControlFlow::Continue(())
    }

    pub fn update_workspace_documents(&self, uri: Url) {
        let uri_fs_path = match uri.to_file_path() {
            Err(_) => return,
            Ok(p) => p,
        };

        debug!("Updating workspace documents: {}", uri_fs_path.display());

        let manifests = Arc::clone(&self.manifests);
        tokio::task::spawn(async move {
            let file_paths = KNOWN_FILE_NAMES
                .iter()
                .map(|file_name| uri_fs_path.join(file_name))
                .collect::<Vec<_>>();

            let file_futs = file_paths
                .iter()
                .map(tokio::fs::read_to_string)
                .collect::<Vec<_>>();
            let file_results = join_all(file_futs).await;

            let mut manifests = manifests.lock().await;
            for (file_path, file_result) in file_paths.into_iter().zip(file_results) {
                parse_and_insert(&mut manifests, file_path, file_result);
            }
        });
    }
}

fn parse_and_insert(
    manifests: &mut HashMap<Url, Manifest>,
    file_path_absolute: PathBuf,
    file_result: Result<String, std::io::Error>,
) {
    trace!("Updating text document '{}'", file_path_absolute.display());

    let file_contents = match file_result {
        Ok(contents) => contents,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                warn!(
                    "Failed to read file at '{}'\n{err}",
                    file_path_absolute.display()
                );
            }
            return;
        }
    };

    // FUTURE: Parse differently depending on file
    match Manifest::parse(file_contents) {
        Err(err) => {
            warn!(
                "Failed to parse file at '{}'\n{err}",
                file_path_absolute.display()
            );
        }
        Ok(manifest) => {
            manifests.insert(
                Url::from_file_path(file_path_absolute).expect("File path passed was not absolute"),
                manifest,
            );
        }
    }
}
