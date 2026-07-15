use std::{sync::Arc, time::Duration};

use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct Broadcaster {
    inner: Mutex<BroadcasterInner>,
}

#[derive(Debug, Clone, Default)]
struct BroadcasterInner {
    clients: Vec<mpsc::Sender<sse::Event>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub action: String,
    pub payload: String,
}

impl Broadcaster {
    /// Constructs new broadcaster and spawns ping loop.
    ///
    /// The ping loop runs on the tauri runtime (not `actix_web::rt`) because
    /// the broadcaster may be created lazily from a tauri command thread,
    /// outside any actix `System`.
    pub fn create() -> Arc<Self> {
        let this = Arc::new(Broadcaster {
            inner: Mutex::new(BroadcasterInner::default()),
        });

        Broadcaster::spawn_ping(Arc::clone(&this));

        this
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast
    /// list if not.
    fn spawn_ping(this: Arc<Self>) {
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                this.remove_stale_clients().await;
            }
        });
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        let clients = self.inner.lock().clients.clone();

        let mut ok_clients = Vec::new();

        for client in clients {
            if client
                .send(sse::Event::Comment("ping".into()))
                .await
                .is_ok()
            {
                ok_clients.push(client.clone());
            }
        }

        self.inner.lock().clients = ok_clients;
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self) -> Sse<InfallibleStream<ReceiverStream<sse::Event>>> {
        let (tx, rx) = mpsc::channel(10);

        let _ = tx
            .send(
                sse::Data::new(
                    json!(Message {
                        action: "connected".to_string(),
                        payload: "".to_string()
                    })
                    .to_string(),
                )
                .into(),
            )
            .await;

        self.inner.lock().clients.push(tx);

        Sse::from_infallible_receiver(rx)
    }

    /// Broadcasts `msg` to all clients.
    pub fn broadcast_sync(&self, msg: Message) {
        let clients = self.inner.lock().clients.clone();

        tauri::async_runtime::spawn(async move {
            for client in clients.iter() {
                // ignore send failures; disconnected clients are swept up by the ping loop
                let _ = client
                    .send(sse::Data::new(json!(msg).to_string()).into())
                    .await;
            }
        });
    }
}
