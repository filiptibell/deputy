use std::ops::ControlFlow;
use std::time::Duration;

use tokio::time;
use tracing::{trace, warn};

use async_lsp::Result;

use super::notifications::*;
use super::*;

#[derive(Debug, Clone, Copy)]
pub struct TickEvent;

#[derive(Debug, Clone, Copy)]
pub enum RateLimitEvent {
    GitHub,
}

impl Backend {
    pub(super) fn spawn_rate_limit(&mut self) {
        let client = self.client.clone();
        let github = self.github.clone();
        tokio::spawn(async move {
            loop {
                let is_rate_limited = github.wait_until_rate_limited_changes().await;
                if is_rate_limited && client.emit(RateLimitEvent::GitHub).is_err() {
                    break;
                }
            }
        });
    }

    pub(super) fn spawn_tick(&mut self) {
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

    pub(super) fn on_event_rate_limit(&mut self, _: RateLimitEvent) -> ControlFlow<Result<()>> {
        warn!("GitHub rate limit was reached");
        let notif = RateLimitNotification::github();
        if self.client.notify::<RateLimitNotification>(notif).is_ok() {
            trace!("GitHub rate limit notification sent");
        }
        ControlFlow::Continue(())
    }

    pub(super) fn on_event_tick(&mut self, _: TickEvent) -> ControlFlow<Result<()>> {
        trace!("tick");
        ControlFlow::Continue(())
    }
}
