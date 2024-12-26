use std::sync::{Arc, Mutex};
use actix_web::{get, web, Responder};
use actix_web_lab::sse;
use std::time::Duration;
use actix_web_lab::__reexports::futures_util::future;
use actix_web_lab::__reexports::serde_json;
use log::debug;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio::time::interval;

#[derive(Serialize, Clone)]
pub enum Job {
    HarvestData,
    ImportData,
    ValidateData,
    Preprocessing,
    PreprocessingClustering,
    PreprocessingLocalTransferPatterns,
    PreprocessingLongDistanceTransferPatterns,
    PreprocessingBorderTransferPatterns,
    ServerStartup
}

#[derive(Serialize, Clone)]
pub enum StatusEvent {
    StartedJob(Job),
    FinishedJob(Job),
}

pub struct StatusBroadcaster {
    inner: Mutex<StatusBroadcasterInner>,
}

#[derive(Debug, Clone, Default)]
pub struct StatusBroadcasterInner {
    clients: Vec<mpsc::Sender<sse::Event>>,
}

impl StatusBroadcaster {
    pub fn create() -> Arc<Self> {
        let broadcaster = Arc::new(StatusBroadcaster {
            inner: Mutex::new(StatusBroadcasterInner::default()),
        });
        StatusBroadcaster::spawn_ping(Arc::clone(&broadcaster));

        broadcaster
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast list if not.
    fn spawn_ping(this: Arc<Self>) {
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                debug!(target: "status endpoint", "Sent PING");
                this.remove_stale_clients().await;
            }
        });
    }

    /// Removes all non-responsive clients from broadcast list.
    async fn remove_stale_clients(&self) {
        let clients = self.inner.lock().unwrap().clients.clone();

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

        self.inner.lock().unwrap().clients = ok_clients;
    }

    /// Registers client with broadcaster, returning an SSE response body.
    pub async fn new_client(&self) -> mpsc::Receiver<sse::Event> {
        debug!(target: "status endpoint", "Client is registering");
        let (tx, rx) = mpsc::channel(10);

        tx.send(sse::Event::Data(sse::Data::new("connected"))).await.unwrap();
        debug!(target: "status endpoint", "Client {:?} successfully registered", tx);
        self.inner.lock().unwrap().clients.push(tx);
        rx
    }

    /// Broadcasts `msg` to all clients.
    pub async fn broadcast(&self, event: StatusEvent) -> Result<(), serde_json::Error> {
        let clients = self.inner.lock().unwrap().clients.clone();
        
        let data = sse::Data::new_json(event)?;

        let send_futures = clients
            .iter()
            .map(|client| client.send(sse::Event::Data(data.clone())));

        // try to send to all clients, ignoring failures
        // disconnected clients will get swept up by `remove_stale_clients`
        let _ = future::join_all(send_futures).await;
        
        Ok(())
    }
}

#[get("/api/v1/status")]
pub(crate) async fn status(state: web::Data<Arc<StatusBroadcaster>>) -> impl Responder {
    let rx = state.new_client().await;
    
    sse::Sse::from_infallible_receiver(rx)
        .with_keep_alive(Duration::from_secs(2))
        .with_retry_duration(Duration::from_secs(5))
}
