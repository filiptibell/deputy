use lsp_types::notification::Notification;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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
}

impl Notification for RateLimitNotification {
    const METHOD: &'static str = "$/ratelimit/reached";
    type Params = Self;
}
