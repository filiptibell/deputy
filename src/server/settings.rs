use std::sync::{Arc, RwLock};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::*;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSettings {
    #[serde(default = "default_true")]
    pub workspace: bool,
}

impl Default for DiagnosticsSettings {
    fn default() -> Self {
        Self { workspace: true }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub diagnostics: DiagnosticsSettings,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct SettingsMap {
    global: Arc<RwLock<Settings>>,
    workspace: Arc<DashMap<Url, Settings>>,
}

impl Default for SettingsMap {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsMap {
    pub fn new() -> Self {
        Self {
            global: Arc::new(RwLock::new(Settings::default())),
            workspace: Arc::new(DashMap::new()),
        }
    }

    pub fn get_global_settings(&self) -> Settings {
        self.global.read().unwrap().clone()
    }

    pub fn get_workspace_settings(&self, workspace_uri: &Url) -> Option<Settings> {
        self.workspace
            .get(workspace_uri)
            .map(|settings| settings.clone())
    }

    pub fn update_global_settings(&self, settings: Settings) {
        debug!("Updating global settings: {:?}", settings);
        let mut global = self.global.write().unwrap();
        *global = settings;
    }

    pub fn update_workspace_settings(&self, workspace_uri: &Url, settings: Settings) {
        debug!(
            "Updating settings for workspace {}: {:?}",
            workspace_uri, settings
        );
        self.workspace.insert(workspace_uri.clone(), settings);
    }

    pub fn get_settings_for_uri(&self, uri: &Url) -> Settings {
        if let Some(workspace_uri) = self.find_workspace_for_uri(uri) {
            if let Some(settings) = self.workspace.get(&workspace_uri) {
                return settings.clone();
            }
        }

        self.global.read().unwrap().clone()
    }

    pub fn is_workspace_diagnostics_enabled(&self) -> bool {
        self.get_global_settings().diagnostics.workspace
    }

    pub fn is_workspace_diagnostics_enabled_for(&self, uri: &Url) -> bool {
        self.get_settings_for_uri(uri).diagnostics.workspace
    }

    fn find_workspace_for_uri(&self, uri: &Url) -> Option<Url> {
        if uri.scheme() != "file" {
            return None;
        }

        self.workspace
            .iter()
            .filter_map(|entry| {
                let workspace_uri = entry.key();
                if workspace_uri.scheme() == "file"
                    && uri.as_str().starts_with(workspace_uri.as_str())
                {
                    Some(workspace_uri.clone())
                } else {
                    None
                }
            })
            .max_by_key(|uri| uri.as_str().len())
    }
}
