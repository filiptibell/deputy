use std::{ops::ControlFlow, time::Duration};

use tokio::time;
use tracing::debug;

use async_lsp::{router::Router, ClientSocket, Result};

use crate::events::*;

pub struct ServerState {
    pub client: ClientSocket,
    pub counter: i32,
}

impl ServerState {
    pub fn new(client: ClientSocket) -> Self {
        let mut this = Self { client, counter: 0 };
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
        debug!("tick");
        self.counter += 1;
        ControlFlow::Continue(())
    }
}
