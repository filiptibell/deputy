use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{debug, warn};

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;

use super::Backend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitNotificationKind {
    GitHub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitNotification {
    pub kind: RateLimitNotificationKind,
    pub value: JsonValue,
}

impl RateLimitNotification {
    pub fn github() -> Self {
        Self {
            kind: RateLimitNotificationKind::GitHub,
            value: JsonValue::Null,
        }
    }

    pub fn value_string(&self) -> Option<String> {
        if let JsonValue::String(s) = &self.value {
            Some(s.to_string())
        } else {
            None
        }
    }
}

impl Notification for RateLimitNotification {
    const METHOD: &'static str = "$/internal_message/rate_limit";
    type Params = Self;
}

impl Backend {
    pub fn watch_rate_limit(&self) {
        let client = self.client.clone();
        let github = self.github.clone();
        tokio::spawn(async move {
            loop {
                let is_rate_limited = github.wait_until_rate_limited_changes().await;
                if is_rate_limited {
                    let notif = RateLimitNotification::github();
                    client
                        .send_notification::<RateLimitNotification>(notif)
                        .await;
                }
            }
        });
    }

    pub async fn on_notified_rate_limit(&self, notif: RateLimitNotification) -> Result<()> {
        if let Some(token) = notif.value_string() {
            self.github.set_auth_token(token);
            debug!("GitHub rate limit notification received - set token");
        } else {
            warn!("GitHub rate limit notification received - no token");
        }
        self.update_all_workspaces().await;
        Ok(())
    }
}
