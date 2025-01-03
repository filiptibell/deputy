#![allow(dead_code)]

use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::oneshot::{channel, Receiver, Sender};
use tower_lsp::lsp_types::*;

type Span = std::ops::Range<usize>;

#[derive(Debug, Clone)]
pub struct Waiting {
    senders: Arc<DashMap<Url, Vec<Sender<()>>>>,
}

impl Waiting {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, uri: Url) -> Receiver<()> {
        let (sender, receiver) = channel();
        self.senders.entry(uri.clone()).or_default().push(sender);
        receiver
    }

    pub fn remove(&self, uri: &Url) {
        self.senders.remove(uri);
    }

    pub fn trigger(&self, uri: Url) {
        if let Some((_, senders)) = self.senders.remove(&uri) {
            for sender in senders {
                let _ = sender.send(());
            }
        }
    }
}

impl Default for Waiting {
    fn default() -> Self {
        Self::new()
    }
}
