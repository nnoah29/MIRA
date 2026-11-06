//! Liaison vers un démon de test : un client attaché, des identifiants d'échange et l'attente d'un `result`.

use std::sync::atomic::{AtomicU64, Ordering};

use mira_protocol::Frame;
use tokio::sync::Mutex;

use crate::support::{TestClient, TestDaemon};

/// Un démon de test et un client attaché ; les échanges se font par trames.
pub struct SocketLink {
    _daemon: TestDaemon,
    pub client: Mutex<TestClient>,
    next_id: AtomicU64,
}

impl SocketLink {
    /// Attache un client au démon.
    pub async fn attach(daemon: TestDaemon) -> Self {
        let mut client = TestClient::connect(&daemon).await;
        client.hello_ok(daemon.token()).await;
        Self {
            _daemon: daemon,
            client: Mutex::new(client),
            next_id: AtomicU64::new(1),
        }
    }

    /// Identifiant d'échange neuf.
    pub fn id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Envoie la trame puis rend l'enveloppe du `result` qui porte l'identifiant ; les autres trames sont ignorées.
    pub async fn result_of(&self, id: u64, frame: Frame) -> String {
        let mut client = self.client.lock().await;
        client.send(frame).await;
        loop {
            match client.recv().await {
                Some(Frame::Result { id: got, envelope }) if got == id => {
                    return envelope.to_string();
                }
                Some(_) => continue,
                None => panic!("connexion fermée avant la réponse"),
            }
        }
    }
}
