//! Client TCP brut : écrit une commande, lit la réponse.

use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use super::response::Response;

/// Client IMAP minimal sur une socket TCP.
#[derive(Debug)]
pub struct RawClient {
    pub(super) reader: BufReader<OwnedReadHalf>,
    pub(super) writer: OwnedWriteHalf,
    pub(super) counter: u32,
    pub greeting: String,
}

impl RawClient {
    /// Se connecte et lit la bannière
    pub async fn connect(address: &str) -> Self {
        let stream = TcpStream::connect(address).await.unwrap();
        let (read, write) = stream.into_split();
        let mut client = Self {
            reader: BufReader::new(read),
            writer: write,
            counter: 0,
            greeting: String::new(),
        };
        client.greeting = client.read_line().await.unwrap_or_default();
        client
    }

    /// Envoie du texte brut
    pub async fn send_raw(&mut self, text: &str) {
        self.writer.write_all(text.as_bytes()).await.unwrap();
    }

    /// Envoie des octets bruts
    pub async fn send_bytes(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).await.unwrap();
    }

    /// Envoie une commande étiquetée et lit sa réponse
    pub async fn command(&mut self, text: &str) -> Response {
        self.counter += 1;
        let tag = format!("a{}", self.counter);
        self.send_raw(&format!("{tag} {text}\r\n")).await;
        self.read_response(&tag).await
    }
}
