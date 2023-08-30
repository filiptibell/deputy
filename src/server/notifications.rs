use std::ops::ControlFlow;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{debug, warn};

use async_lsp::Result;
use lsp_types::notification::Notification;

use super::Server;

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

impl Server {
    pub(super) fn on_notified_rate_limit(
        &mut self,
        notif: RateLimitNotification,
    ) -> ControlFlow<Result<()>> {
        if let Some(token) = notif.value_string() {
            self.github.set_auth_token(token);
            debug!("GitHub rate limit notification received - set token");
        } else {
            warn!("GitHub rate limit notification received - no token");
        }
        self.update_all_workspaces();
        ControlFlow::Continue(())
    }
}
