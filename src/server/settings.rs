use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
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
        }
    }

    pub fn get_global_settings(&self) -> Settings {
        self.global.read().unwrap().clone()
    }

    pub fn update_global_settings(&self, settings: Settings) {
        debug!("Updating global settings: {:?}", settings);
        let mut global = self.global.write().unwrap();
        *global = settings;
    }

    pub fn is_workspace_diagnostics_enabled(&self) -> bool {
        self.get_global_settings().diagnostics.workspace
    }
}
