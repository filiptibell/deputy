use std::{collections::HashMap, ops::ControlFlow, path::PathBuf, sync::Arc};

use futures::future::join_all;
use semver::Version;
use tracing::{debug, info, trace, warn};

use async_lsp::{ClientSocket, Result};

use lsp_types::notification::{Progress, PublishDiagnostics};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString, ProgressParams, ProgressParamsValue,
    PublishDiagnosticsParams, Range, Url, WorkDoneProgress, WorkDoneProgressBegin,
    WorkDoneProgressEnd,
};

use crate::github::*;
use crate::manifest::*;
use crate::util::*;

use super::super::actions::*;
use super::super::document::*;
use super::super::*;

const KNOWN_FILE_NAMES: &[&str] = &["aftman.toml", "foreman.toml", "wally.toml"];

impl Backend {
    pub fn update_document(
        &self,
        uri: Url,
        version: i32,
        contents: String,
    ) -> ControlFlow<Result<()>> {
        let file_path = match uri.to_file_path() {
            Err(_) => return ControlFlow::Continue(()),
            Ok(path) => path,
        };

        debug!("Updating document: {}", file_path.display());

        let client = self.client.clone();
        let github = self.github.clone();
        let documents = Arc::clone(&self.documents);
        tokio::task::spawn(async move {
            let mut documents = documents.lock().await;
            if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                if KNOWN_FILE_NAMES.contains(&file_name) {
                    parse_and_insert(
                        client,
                        github,
                        &mut documents,
                        file_path,
                        Some(version),
                        Ok(contents),
                    );
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
        let documents = Arc::clone(&self.documents);
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

            let mut documents = documents.lock().await;
            for (file_path, file_result) in file_paths.into_iter().zip(file_results) {
                parse_and_insert(
                    client.clone(),
                    github.clone(),
                    &mut documents,
                    file_path,
                    Some(0),
                    file_result,
                );
            }
        });
    }

    pub fn update_all_workspaces(&self) {
        info!("Updating all workspaces");
        for (_, uri) in &self.workspace_folders {
            self.update_workspace_documents(uri.clone());
        }
    }
}

fn parse_and_insert(
    client: ClientSocket,
    github: GithubWrapper,
    documents: &mut HashMap<Url, Document>,
    file_path_absolute: PathBuf,
    file_version: Option<i32>,
    file_result: Result<String, std::io::Error>,
) {
    trace!("Updating text document '{}'", file_path_absolute.display());

    let progress_token = file_path_absolute.to_string_lossy().to_string();

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

            documents.insert(
                uri.clone(),
                Document {
                    uri: uri.clone(),
                    version: file_version,
                    manifest,
                },
            );

            tokio::task::spawn(async move {
                let diags = tools
                    .iter()
                    .map(|(tool, range)| diagnose_tool_version(&github, &uri, tool, range))
                    .collect::<Vec<_>>();

                let _ = client.notify::<Progress>(ProgressParams {
                    token: NumberOrString::String(progress_token.clone()),
                    value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                        WorkDoneProgressBegin {
                            cancellable: Some(false),
                            title: String::from("Fetching latest versions"),
                            ..Default::default()
                        },
                    )),
                });

                let mut diags = join_all(diags).await;
                for diag in diags.drain(..).flatten() {
                    diagnostics.push(diag);
                }

                let _ = client.notify::<Progress>(ProgressParams {
                    token: NumberOrString::String(progress_token.clone()),
                    value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(
                        WorkDoneProgressEnd::default(),
                    )),
                });

                let _ = client.notify::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: uri.clone(),
                    diagnostics,
                    version: file_version,
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
    uri: &Url,
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

    let latest_tag = latest.tag_name.trim_start_matches('v');
    let latest_version = match Version::parse(latest_tag) {
        Err(_) => return None,
        Ok(v) => v,
    };

    if latest_version > spec.version {
        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: uri.clone(),
            source_text: tool.val_text.to_string(),
            version_current: spec.version.to_string(),
            version_latest: latest_version.to_string(),
        };
        Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!(
                "A newer tool version is available.\
                \nThe latest version is `{latest_version}`"
            ),
            severity: Some(DiagnosticSeverity::INFORMATION),
            data: Some(metadata.into()),
            ..Default::default()
        })
    } else {
        None
    }
}
