//! Client brut de test : écrit des trames ou des octets, enregistre tout ce que le démon renvoie.

mod calls;
mod recv;

use std::path::Path;

use mira_daemon::transport::ByteStream;
use mira_protocol::{Frame, MAX_FRAME_BYTES, encode};
use tokio::io::AsyncWriteExt;

pub use calls::{hello_with, valid_hello};
pub use recv::Recv;

use super::TestDaemon;

/// Connexion de test avec un tampon de lecture et le journal des octets reçus.
pub struct TestClient {
    stream: Box<dyn ByteStream>,
    buffer: Vec<u8>,
    received: Vec<u8>,
    next_id: u64,
}

impl TestClient {
    /// Se connecte à la socket Unix du démon de test.
    pub async fn connect(daemon: &TestDaemon) -> Self {
        Self::connect_unix(daemon.socket_path()).await
    }

    /// Se connecte à cette socket Unix.
    pub async fn connect_unix(path: &Path) -> Self {
        let stream = tokio::net::UnixStream::connect(path).await.unwrap();
        Self::over(Box::new(stream))
    }

    /// Enveloppe un flux déjà connecté.
    pub fn over(stream: Box<dyn ByteStream>) -> Self {
        Self {
            stream,
            buffer: Vec::new(),
            received: Vec::new(),
            next_id: 1,
        }
    }

    /// Envoie une trame encodée.
    pub async fn send(&mut self, frame: Frame) {
        let bytes = encode(&frame, MAX_FRAME_BYTES).unwrap();
        self.send_raw(&bytes).await;
    }

    /// Envoie ces octets tels quels ; une connexion déjà fermée par le démon n'est pas une erreur.
    pub async fn send_raw(&mut self, bytes: &[u8]) {
        let _ = self.stream.write_all(bytes).await;
    }

    /// Tous les octets reçus du démon depuis la connexion.
    pub fn received_bytes(&self) -> &[u8] {
        &self.received
    }
}
