//! Lecture et écriture sur la socket d'une session.

use std::io;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

/// Socket d'une session, avec lecture tamponnée.
#[derive(Debug)]
pub(crate) struct Transport {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl Transport {
    /// Enveloppe une connexion acceptée
    pub(crate) fn new(stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();
        Self {
            reader: BufReader::new(read),
            writer: write,
        }
    }

    /// Ajoute une ligne (fin de ligne comprise) au tampon ; renvoie le nombre d'octets lus
    pub(crate) async fn read_line_into(&mut self, buffer: &mut Vec<u8>) -> io::Result<usize> {
        self.reader.read_until(b'\n', buffer).await
    }

    /// Lit exactement `count` octets
    pub(crate) async fn read_exact_bytes(&mut self, count: usize) -> io::Result<Vec<u8>> {
        let mut data = vec![0; count];
        self.reader.read_exact(&mut data).await?;
        Ok(data)
    }

    /// Envoie des octets au client
    pub(crate) async fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes).await?;
        self.writer.flush().await
    }
}
