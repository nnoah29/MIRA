//! Handle du faux serveur d'IA : écoute sur 127.0.0.1, port éphémère.

use std::fmt;
use std::io;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::listener::accept_loop;
use super::state::AiShared;

/// Faux serveur d'IA ; l'écoute s'arrête quand le handle est détruit ou après `stop`.
pub struct FakeAiServer {
    pub(super) addr: SocketAddr,
    pub(super) shared: AiShared,
    task: JoinHandle<()>,
}

impl FakeAiServer {
    /// Démarre le serveur sur un port éphémère de la boucle locale
    pub async fn start() -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let shared = AiShared::default();
        let task = tokio::spawn(accept_loop(listener, shared.clone()));
        Ok(Self { addr, shared, task })
    }

    /// Arrête l'écoute et coupe les connexions en cours, y compris celles qui attendent
    pub async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl fmt::Debug for FakeAiServer {
    /// Affiche l'adresse seulement
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FakeAiServer")
            .field("addr", &self.addr)
            .finish()
    }
}

impl Drop for FakeAiServer {
    /// Arrête la tâche d'écoute à la destruction du handle
    fn drop(&mut self) {
        self.task.abort();
    }
}
