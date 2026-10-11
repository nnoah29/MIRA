//! Écouteur de boucle locale qui joue le rôle du moteur et reçoit la redirection.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Écouteur éphémère sur 127.0.0.1.
pub struct Loopback {
    listener: TcpListener,
    port: u16,
}

impl Loopback {
    /// Ouvre l'écouteur sur un port éphémère
    pub async fn bind() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        Self { listener, port }
    }

    /// Adresse de redirection que le moteur fournirait au serveur
    pub fn redirect_uri(&self) -> String {
        format!("http://127.0.0.1:{}/callback", self.port)
    }

    /// Attend une redirection et renvoie la cible de la requête GET reçue
    pub fn expect_redirect(self) -> JoinHandle<String> {
        tokio::spawn(async move { serve_one(self.listener).await })
    }
}

/// Accepte une connexion, répond 200 et renvoie le chemin avec sa requête
async fn serve_one(listener: TcpListener) -> String {
    let (stream, _) = listener.accept().await.unwrap();
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await.unwrap();
    let mut header = String::new();
    while reader.read_line(&mut header).await.unwrap() > 2 {
        header.clear();
    }
    let reply = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
    reader.get_mut().write_all(reply.as_bytes()).await.unwrap();
    request_line.split(' ').nth(1).unwrap().to_string()
}
