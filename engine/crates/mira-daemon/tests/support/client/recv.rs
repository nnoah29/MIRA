//! Lecture de trames côté client de test, sûre à interrompre par un délai.

use std::time::Duration;

use mira_protocol::{Frame, decode};
use tokio::io::AsyncReadExt;

use super::TestClient;

/// Résultat d'une attente : une trame, la fermeture, ou le silence.
#[derive(Debug)]
pub enum Recv {
    Frame(Frame),
    Closed,
    Quiet,
}

impl TestClient {
    /// Extrait une trame complète du tampon, s'il y en a une.
    fn take_frame(&mut self) -> Option<Frame> {
        let header: [u8; 4] = self.buffer.get(..4)?.try_into().ok()?;
        let length = u32::from_be_bytes(header) as usize;
        let body = self.buffer.get(4..4 + length)?;
        let frame = decode(body).unwrap();
        self.buffer.drain(..4 + length);
        Some(frame)
    }

    /// Attend une trame au plus `wait` ; les octets reçus sont conservés même si le délai coupe l'attente.
    pub async fn recv_for(&mut self, wait: Duration) -> Recv {
        loop {
            if let Some(frame) = self.take_frame() {
                return Recv::Frame(frame);
            }
            let mut chunk = [0u8; 4096];
            match tokio::time::timeout(wait, self.stream.read(&mut chunk)).await {
                Err(_) => return Recv::Quiet,
                Ok(Ok(0) | Err(_)) => return Recv::Closed,
                Ok(Ok(read)) => {
                    self.buffer.extend_from_slice(&chunk[..read]);
                    self.received.extend_from_slice(&chunk[..read]);
                }
            }
        }
    }

    /// Attend une trame cinq secondes ; `None` si le démon a fermé la connexion.
    pub async fn recv(&mut self) -> Option<Frame> {
        match self.recv_for(Duration::from_secs(5)).await {
            Recv::Frame(frame) => Some(frame),
            Recv::Closed => None,
            Recv::Quiet => panic!("silence du démon"),
        }
    }
}
