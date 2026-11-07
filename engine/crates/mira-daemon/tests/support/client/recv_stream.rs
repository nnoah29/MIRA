//! Lecture d'un flux précis parmi les trames reçues, les autres échanges étant ignorés.

use std::time::Duration;

use mira_protocol::Frame;
use tokio::io::AsyncWriteExt;

use super::{Recv, TestClient};

/// Identifiant du flux d'une trame `event` ou `chat_delta`.
fn frame_stream(frame: &Frame) -> Option<u64> {
    match frame {
        Frame::Event { id, .. } | Frame::ChatDelta { id, .. } => Some(*id),
        _ => None,
    }
}

impl TestClient {
    /// Prochaine trame du flux `id` ; `None` après son `end` ou à la fermeture ; les autres flux sont ignorés.
    pub async fn recv_stream(&mut self, id: u64) -> Option<Frame> {
        loop {
            match self.recv_for(Duration::from_secs(60)).await {
                Recv::Frame(Frame::End { id: got, .. }) if got == id => return None,
                Recv::Frame(frame) if frame_stream(&frame) == Some(id) => return Some(frame),
                Recv::Frame(_) | Recv::Quiet => continue,
                Recv::Closed => return None,
            }
        }
    }

    /// Ferme seulement le côté écriture : le démon voit la fin du flux mais peut encore répondre.
    pub async fn half_close(&mut self) {
        let _ = self.stream.shutdown().await;
    }
}
