use std::{sync::Arc, time::Duration};

use actix_web::rt::time::interval;
use actix_web_lab::{
    sse::{self, Sse},
    util::InfallibleStream,
};
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use serde::{Serialize, Deserialize};
use serde_json::json;

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
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

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

        tx.send(sse::Data::new(
        	json!(Message {
				action: "connected".to_string(),
				payload: "".to_string()
			}).to_string()
        ).into()).await.unwrap();

        self.inner.lock().clients.push(tx);

        Sse::from_infallible_receiver(rx)
    }

    /// Broadcasts `msg` to all clients.
    // pub async fn broadcast(&self, msg: &str) {
    //     let clients = self.inner.lock().clients.clone();

    //     let send_futures = clients
    //         .iter()
    //         .map(|client| client.send(sse::Data::new(msg).into()));

    //     // try to send to all clients, ignoring failures
    //     // disconnected clients will get swept up by `remove_stale_clients`
    //     let _ = future::join_all(send_futures).await;
    // }

    pub fn broadcast_sync(&self, msg: Message) {
        let clients = self.inner.lock().clients.clone();
        let msg_copy = msg.to_owned();

        tauri::async_runtime::spawn(async move {
            for client in clients.iter() {
                let _ = client.send(sse::Data::new(json!(msg_copy).to_string()).into()).await; // Ignoring potential errors
            }
        });
    }
}
