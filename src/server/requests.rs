use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::time::sleep;
use tracing::{debug, trace};

use tower_lsp::lsp_types::request::Request;

use super::Server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitKind {
    GitHub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRequest {
    pub kind: RateLimitKind,
    pub value: JsonValue,
}

impl RateLimitRequest {
    pub fn github() -> Self {
        Self {
            kind: RateLimitKind::GitHub,
            value: JsonValue::Null,
        }
    }
}

impl Request for RateLimitRequest {
    const METHOD: &'static str = "$/internal_request/rate_limit";
    type Params = Self;
    type Result = RateLimitResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    pub kind: RateLimitKind,
    pub value: JsonValue,
}

impl RateLimitResponse {
    pub fn value_string(&self) -> Option<String> {
        if let JsonValue::String(s) = &self.value {
            Some(s.to_string())
        } else {
            None
        }
    }
}

impl Server {
    pub fn watch_rate_limit(&self) {
        let client = self.client.clone();
        let github = self.clients.github.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(2)).await;
                trace!("Checking rate limits");
                if github.is_rate_limited() {
                    let notif = RateLimitRequest::github();
                    let response = client.send_request::<RateLimitRequest>(notif).await;
                    if let Ok(res) = response {
                        if let Some(token) = res.value_string() {
                            github.set_auth_token(token);
                            client
                                .workspace_diagnostic_refresh()
                                .await
                                .expect("Server should have been initialized");
                            debug!("GitHub rate limit response received - set token");
                        } else {
                            debug!("GitHub rate limit response received - no token");
                        }
                    }
                    sleep(Duration::from_secs(240)).await;
                }
            }
        });
    }
}
