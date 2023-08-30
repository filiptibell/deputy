use std::{collections::HashMap, ops::ControlFlow, path::PathBuf, sync::Arc, time::Duration};

use futures::future::join_all;
use semver::Version;
use tokio::{sync::Mutex as AsyncMutex, time};
use tracing::{debug, trace, warn};

use async_lsp::{router::Router, ClientSocket, Result};

use lsp_types::notification::PublishDiagnostics;
use lsp_types::{Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams, Range, Url};

use super::events::*;
use crate::github::*;
use crate::manifest::*;
use crate::util::*;

const KNOWN_FILE_NAMES: &[&str] = &["aftman.toml", "foreman.toml", "wally.toml"];

pub struct Server {
    pub client: ClientSocket,
    pub github: GithubWrapper,
    pub manifests: Arc<AsyncMutex<HashMap<Url, Manifest>>>,
}

impl Server {
    pub fn new(client: ClientSocket) -> Self {
        let mut this = Self {
            client,
            github: GithubWrapper::new(),
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
        trace!("tick");
        ControlFlow::Continue(())
    }

    pub fn update_document(&self, uri: Url, contents: String) -> ControlFlow<Result<()>> {
        let file_path = match uri.to_file_path() {
            Err(_) => return ControlFlow::Continue(()),
            Ok(path) => path,
        };

        debug!("Updating document: {}", file_path.display());

        let client = self.client.clone();
        let github = self.github.clone();
        let manifests = Arc::clone(&self.manifests);
        tokio::task::spawn(async move {
            let mut manifests = manifests.lock().await;
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                if KNOWN_FILE_NAMES.contains(&file_name) {
                    parse_and_insert(client, github, &mut manifests, file_path, Ok(contents));
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

        let client = self.client.clone();
        let github = self.github.clone();
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
                parse_and_insert(
                    client.clone(),
                    github.clone(),
                    &mut manifests,
                    file_path,
                    file_result,
                );
            }
        });
    }
}

fn parse_and_insert(
    client: ClientSocket,
    github: GithubWrapper,
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

    // FUTURE: Parse differently depending on file?
    match Manifest::parse(file_contents) {
        Err(err) => {
            // FUTURE: Surface parsing error as diagnostic
            warn!(
                "Failed to parse file at '{}'\n{err}",
                file_path_absolute.display()
            );
        }
        Ok(manifest) => {
            let uri =
                Url::from_file_path(file_path_absolute).expect("File path passed was not absolute");

            let tools = manifest
                .tools_map
                .tools
                .iter()
                .map(|tool| {
                    let range = offset_range_to_range(&manifest.source, tool.val_span.clone());
                    (tool.clone(), range)
                })
                .collect::<Vec<_>>();

            let mut diagnostics = Vec::new();
            for (tool, range) in &tools {
                if let Some(diag) = diagnose_tool_spec(tool, range) {
                    diagnostics.push(diag);
                }
            }

            manifests.insert(uri.clone(), manifest);

            tokio::task::spawn(async move {
                for (tool, range) in tools {
                    if let Some(diag) = diagnose_tool_version(&github, &tool, &range).await {
                        diagnostics.push(diag);
                    }
                }

                let _ = client.notify::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics,
                    version: None,
                });
            });
        }
    }
}

fn diagnose_tool_spec(tool: &ManifestTool, range: &Range) -> Option<Diagnostic> {
    if let Err(err) = tool.spec() {
        Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: err.to_string(),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        })
    } else {
        None
    }
}

async fn diagnose_tool_version(
    github: &GithubWrapper,
    tool: &ManifestTool,
    range: &Range,
) -> Option<Diagnostic> {
    let spec = match tool.spec() {
        Err(_) => return None,
        Ok(s) => s,
    };
    let latest = match github.get_latest_release(spec.author, spec.name).await {
        Err(_) => return None,
        Ok(l) => l,
    };
    let (version_current, version_latest) = match (
        Version::parse(spec.version.trim_start_matches('v')),
        Version::parse(latest.tag_name.trim_start_matches('v')),
    ) {
        (Err(_), _) => return None,
        (_, Err(_)) => return None,
        (Ok(version_current), Ok(version_latest)) => (version_current, version_latest),
    };

    if version_current != version_latest {
        Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!(
                "A newer tool version is available.\
                \nThe latest version is `{version_latest}`"
            ),
            severity: Some(DiagnosticSeverity::INFORMATION),
            ..Default::default()
        })
    } else {
        None
    }
}
